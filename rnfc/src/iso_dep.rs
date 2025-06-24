use rnfc_traits::iso_dep::Reader as IsoDepReader;
use rnfc_traits::iso14443a::{Error as _, Reader as Iso14443aReader};
use rnfc_traits::iso14443a_ll::ErrorKind;

pub const ATS_MAX_LEN: usize = 32; // TODO??

const FSC_MAX: usize = 256;
const FSC_MAX_WITHOUT_CRC: usize = FSC_MAX - 2;

pub struct IsoDepA<T: Iso14443aReader> {
    card: T,

    /// Max frame size we can send to the card, including header and crc.
    /// Ex: if header is 1 byte (no CID/NAD) then max INF field size is FSC-3.
    fsc: usize,

    /// Start-up frame guard time, in units of 1/Fc
    #[allow(unused)] // TODO implement support in lower layers.
    sfgt_1fc: u32,

    /// Framr Waiting Time, in units of 1/Fc
    fwt_1fc: u32,

    /// Block count spin bit: 0 or 1
    block_num: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    Iso14443a(E),
    Protocol,
    Communication,
    TxFrameTooBig,
    RxFrameTooBig,
}

// Divide by 2 so it fits in u8, saving some space
const FS_DIV_2_TABLE: [u8; 9] = [
    16 / 2,
    24 / 2,
    32 / 2,
    40 / 2,
    48 / 2,
    64 / 2,
    96 / 2,
    128 / 2,
    128, // 256 / 2
];

const RATS_TIMEOUT_1FC: u32 = 65536;

impl<T: Iso14443aReader> IsoDepA<T>
where
    T::Error: crate::fmt::Format,
{
    pub async fn new(mut card: T) -> Result<Self, Error<T::Error>> {
        // RATS
        let req = [0xe0, 0x80];
        let mut res = [0; ATS_MAX_LEN];
        let res_len = match card.transceive(&req, &mut res, RATS_TIMEOUT_1FC).await {
            Ok(len) => len,
            Err(e) => {
                warn!("Trx RATS failed: {:?}", e);
                return Err(Error::Iso14443a(e));
            }
        };
        let ats = &res[..res_len];

        let mut fsci = 2;
        let mut sfgi = 0;
        let mut fwi = 4;

        if ats.len() >= 2 {
            let t0 = ats[1];
            // format byte present.
            fsci = (t0 & 0xF) as usize;
            if t0 & 0x20 != 0 {
                let tb_idx = if t0 & 0x10 != 0 { 3 } else { 2 };
                if let Some(tb) = ats.get(tb_idx) {
                    sfgi = tb & 0x0f;
                    fwi = tb >> 4;
                }
            }
        }

        if fsci >= FS_DIV_2_TABLE.len() {
            warn!("FSCI too high");
            return Err(Error::Protocol);
        }
        let fsc = FS_DIV_2_TABLE[fsci] as usize * 2;

        // SFGT = (256 x 16 / fc) x 2^SFGI
        let sfgt_1fc = (256 * 16) << sfgi;
        // FWT = (256 x 16 / fc) x 2^FWI
        let fwt_1fc = (256 * 16) << fwi;

        debug!("fsc= {}, sfgt={}/fc, fwt={}/fc", fsc, sfgt_1fc, fwt_1fc);

        Ok(Self {
            card,
            fsc,
            sfgt_1fc,
            fwt_1fc,
            block_num: 0,
        })
    }

    pub fn inner(&self) -> &T {
        &self.card
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.card
    }

    pub async fn deselect(&mut self) -> Result<(), Error<T::Error>> {
        let tx_buf = [0xC2];
        let mut rx_buf = [0; 1];

        let rx_len = self
            .card
            .transceive(&tx_buf, &mut rx_buf, self.fwt_1fc)
            .await
            .map_err(Error::Iso14443a)?;
        if rx_len != 1 || rx_buf != [0xC2] {
            return Err(Error::Protocol);
        }

        Ok(())
    }
}

impl<T: Iso14443aReader> IsoDepReader for IsoDepA<T>
where
    T::Error: crate::fmt::Format,
{
    type Error = Error<T::Error>;

    async fn transceive(&mut self, mut tx: &[u8], mut rx: &mut [u8]) -> Result<usize, Self::Error> {
        let mut tx_buf = [0; FSC_MAX_WITHOUT_CRC];
        let mut rx_buf = [0; FSC_MAX_WITHOUT_CRC];

        enum Send {
            Data,
            Ack,
            Nak,
            Wtx(u8),
        }
        let mut send = Send::Data;

        let max_n = self.fsc - 3;
        let mut rx_total = 0;
        let mut rx_chaining = false;
        let mut retries = 0;

        loop {
            let mut fwt = self.fwt_1fc;
            let tx_len = match send {
                Send::Data => {
                    let n = tx.len().min(max_n);
                    let more_blocks = n != tx.len();
                    tx_buf[0] = 0x02 | self.block_num | (more_blocks as u8) << 4;
                    tx_buf[1..][..n].copy_from_slice(&tx[..n]);
                    1 + n
                }
                Send::Wtx(mul) => {
                    fwt *= mul as u32;
                    tx_buf[0] = 0xF2;
                    tx_buf[1] = mul;
                    2
                }
                Send::Ack => {
                    tx_buf[0] = 0xa2 | self.block_num;
                    1
                }
                Send::Nak => {
                    tx_buf[0] = 0xb2 | self.block_num;
                    1
                }
            };

            let res = self.card.transceive(&tx_buf[..tx_len], &mut rx_buf, fwt).await;

            send = match res {
                Err(e) => {
                    warn!("isodep: got error {:?}", e);
                    match e.kind() {
                        ErrorKind::Timeout | ErrorKind::Corruption => {
                            retries += 1;
                            if retries >= 10 {
                                return Err(Error::Communication);
                            }
                            match rx_chaining {
                                true => Send::Ack,
                                false => Send::Nak,
                            }
                        }
                        _ => return Err(Error::Iso14443a(e)),
                    }
                }
                Ok(rx_len) => {
                    if rx_len == 0 {
                        warn!("isodep: received zero len data");
                        return Err(Error::Protocol);
                    }

                    retries = 0;

                    let rx_pcb = rx_buf[0]; // protocol control byte (aka header)
                    match rx_pcb {
                        // I-block
                        0x02 | 0x03 | 0x12 | 0x13 => {
                            let rx_inf_len = rx_len - 1;
                            if rx_inf_len > rx.len() {
                                return Err(Error::RxFrameTooBig);
                            }

                            rx[..rx_inf_len].copy_from_slice(&rx_buf[1..][..rx_inf_len]);
                            rx = &mut rx[rx_inf_len..];
                            rx_total += rx_inf_len;

                            // spin the spinny bit
                            self.block_num ^= 1;

                            // last block of chaining.
                            if rx_pcb & 0x10 == 0 {
                                return Ok(rx_total);
                            }

                            rx_chaining = true;
                            Send::Ack
                        }
                        0xa2 | 0xa3 => {
                            // if block number is right, advance to next chaining block.
                            if rx_pcb & 1 == self.block_num {
                                if tx.len() <= max_n {
                                    warn!("isodep: got ack on last chaining block");
                                    return Err(Error::Protocol);
                                }
                                tx = &tx[max_n..];

                                // spin the spinny bit
                                self.block_num ^= 1;
                            }
                            Send::Data
                        }
                        // S-block Waiting Time Extension - WTX
                        0xF2 => {
                            if rx_len != 2 {
                                warn!("isodep: invalid S(WTX) len {}", rx_len);
                                return Err(Error::Protocol);
                            }
                            Send::Wtx(rx_buf[1] & 0x3F)
                        }
                        _ => {
                            warn!("unknown rx pcb {:02x}", rx_pcb);
                            return Err(Error::Protocol);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::vec::Vec;

    use hex_literal::hex;
    use rnfc_traits::iso_dep::Reader;
    use rnfc_traits::iso14443a::Reader as Iso14443aReader;
    use rnfc_traits::iso14443a_ll::ErrorKind;

    use super::*;

    struct MockReader {
        expected: Vec<(&'static [u8], Result<&'static [u8], ErrorKind>)>,
        pos: usize,
    }

    macro_rules! mock {
        (@res $rx:literal) => {
            Ok(&hex_literal::hex!($rx))
        };
        (@res timeout) => {
            Err(ErrorKind::Timeout)
        };
        ($($tx:literal => $rx:tt,)*) => {
            MockReader {
                expected: vec![
                    $((&hex_literal::hex!($tx), mock!(@res $rx)),)*
                ],
                pos: 0,
            }
        };
    }

    impl Iso14443aReader for MockReader {
        type Error = ErrorKind;

        async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], _: u32) -> Result<usize, Self::Error> {
            if self.pos >= self.expected.len() {
                panic!("unexpected transceive!\n         got: {:02x?}", tx);
            }

            let (expected_tx, expected_rx) = self.expected[self.pos];
            if tx != expected_tx {
                panic!(
                    "unexpected tx!\n    expected: {:02x?}\n         got: {:02x?}",
                    expected_tx, tx
                );
            }

            self.pos += 1;
            match expected_rx {
                Ok(expected_rx) => {
                    rx[..expected_rx.len()].copy_from_slice(expected_rx);
                    Ok(expected_rx.len())
                }
                Err(e) => Err(e),
            }
        }

        fn atqa(&self) -> [u8; 2] {
            todo!()
        }

        fn sak(&self) -> u8 {
            todo!()
        }

        fn uid(&self) -> &[u8] {
            todo!()
        }
    }

    macro_rules! trx {
        ($x:expr, $tx:literal => $rx:literal) => {
            let mut buf = [0u8; 256];
            let n = $x.transceive(&hex!($tx), &mut buf).await.unwrap();
            let rx = &buf[..n];
            let expected_rx = &hex!($rx)[..];
            if rx != expected_rx {
                panic!(
                    "unexpected rx!\n    expected: {:02x?}\n         got: {:02x?}",
                    expected_rx, rx
                );
            }
        };
        ($x:expr, $tx:literal => $err:expr) => {
            let mut buf = [0u8; 256];
            let res = $x.transceive(&hex!($tx), &mut buf).await;
            assert_eq!(res, Err($err));
        };
    }
    #[test_log::test(tokio::test)]
    async fn test_init() {
        // Nothing present.
        let mock = mock!(
            "e0 80" => "01",
        );
        let x = IsoDepA::new(mock).await.unwrap();
        assert_eq!(x.fsc, 32);
        assert_eq!(x.sfgt_1fc, 256 * 16);
        assert_eq!(x.fwt_1fc, 256 * 16 * 16);

        // T0 present, nothing else.
        let mock = mock!(
            "e0 80" => "02 05",
        );
        let x = IsoDepA::new(mock).await.unwrap();
        assert_eq!(x.fsc, 64);
        assert_eq!(x.sfgt_1fc, 256 * 16);
        assert_eq!(x.fwt_1fc, 256 * 16 * 16);

        // TA not present, TB present
        let mock = mock!(
            "e0 80" => "05 67 81 02 80",
        );
        let x = IsoDepA::new(mock).await.unwrap();
        assert_eq!(x.fsc, 128);
        assert_eq!(x.sfgt_1fc, 8192);
        assert_eq!(x.fwt_1fc, 1048576);

        // TA present, TB present
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
        );
        let x = IsoDepA::new(mock).await.unwrap();
        assert_eq!(x.fsc, 128);
        assert_eq!(x.sfgt_1fc, 8192);
        assert_eq!(x.fwt_1fc, 1048576);
    }

    // B.2.1 Exchange of I-blocks. Scenario 1
    #[test_log::test(tokio::test)]
    async fn test_exchange_iblocks() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 12 34" => "02 56 78",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "12 34" => "56 78");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.2.2 Request for waiting time extension. Scenario 2
    #[test_log::test(tokio::test)]
    async fn test_request_wtx() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 12 34" => "f2 c1",
            "f2 01" => "f2 c1",
            "f2 01" => "02 56 78",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "12 34" => "56 78");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.2.4 Chaining - Scenario 4 PCD uses chaining
    // Figure 22 — Chaining
    #[test_log::test(tokio::test)]
    async fn test_pcd_chaining() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "12 00 11 22 33 44 55 66" => "a2",
            "13 77 88 99 aa bb cc dd" => "a3",
            "02 ee ff" => "02 cc dd",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff" => "cc dd");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.2.4 Chaining - Scenario 5 PICC uses chaining
    #[test_log::test(tokio::test)]
    async fn test_picc_chaining() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 12 34" => "12 00 11 22 33 44 55 66",
            "a3" => "13 77 88 99 aa bb cc dd",
            "a2" => "02 ee ff",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "12 34" => "00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.3 Error handling B.3.1 Exchange of I-blocks - Scenario 6 Start of protocol
    #[test_log::test(tokio::test)]
    async fn test_error_iblock_start() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => timeout,
            "b2" => "a3",
            "02 11 22" => "02 33 44",
            "03 55 66" => "03 77 88",
            "02 99 aa" => "02 bb cc",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
        trx!(x, "99 aa" => "bb cc");
    }

    // B.3 Error handling B.3.1 Exchange of I-blocks - Scenario 7
    #[test_log::test(tokio::test)]
    async fn test_error_iblock() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => "02 33 44",
            "03 55 66" => timeout,
            "b3" => "a2",
            "03 55 66" => "03 77 88",
            "02 99 aa" => "02 bb cc",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
        trx!(x, "99 aa" => "bb cc");
    }

    // B.3 Error handling B.3.1 Exchange of I-blocks - Scenario 8
    #[test_log::test(tokio::test)]
    async fn test_error_iblock_2() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => timeout,
            "b2" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.1 Exchange of I-blocks - Scenario 9
    #[test_log::test(tokio::test)]
    async fn test_error_iblock_3() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => timeout,
            "b2" => timeout,
            "b2" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.2 Request for waiting time extension - Scenario 10
    #[test_log::test(tokio::test)]
    async fn test_error_wtx_1() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => timeout,
            "b2" => "f2 c1",
            "f2 01" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.2 Request for waiting time extension - Scenario 11
    #[test_log::test(tokio::test)]
    async fn test_error_wtx_2() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => timeout,
            "b2" => timeout,
            "b2" => "f2 c1",
            "f2 01" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.2 Request for waiting time extension - Scenario 12
    #[test_log::test(tokio::test)]
    async fn test_error_wtx_3() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => "f2 c1",
            "f2 01" => timeout,
            "b2" => "f2 c1",
            "f2 01" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.2 Request for waiting time extension - Scenario 13
    #[test_log::test(tokio::test)]
    async fn test_error_wtx_4() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => "f2 c1",
            "f2 01" => timeout,
            "b2" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.2 Request for waiting time extension - Scenario 14
    #[test_log::test(tokio::test)]
    async fn test_error_wtx_5() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 11 22" => "f2 c1",
            "f2 01" => timeout,
            "b2" => timeout,
            "b2" => "02 33 44",
            "03 55 66" => "03 77 88",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        trx!(x, "11 22" => "33 44");
        trx!(x, "55 66" => "77 88");
    }

    // B.3 Error handling B.3.4 Chaining - Scenario 16
    #[test_log::test(tokio::test)]
    async fn test_error_pcd_chaining_1() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "12 00 11 22 33 44 55 66" => timeout,
            "b2" => "a2",
            "13 77 88 99 aa bb cc dd" => "a3",
            "02 ee ff" => "02 cc dd",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff" => "cc dd");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.3 Error handling B.3.4 Chaining - Scenario 17
    #[test_log::test(tokio::test)]
    async fn test_error_pcd_chaining_2() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "12 00 11 22 33 44 55 66" => "a2",
            "13 77 88 99 aa bb cc dd" => timeout,
            "b3" => "a2",
            "13 77 88 99 aa bb cc dd" => "a3",
            "02 ee ff" => "02 cc dd",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff" => "cc dd");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.3 Error handling B.3.4 Chaining - Scenario 18
    #[test_log::test(tokio::test)]
    async fn test_error_pcd_chaining_3() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "12 00 11 22 33 44 55 66" => timeout,
            "b2" => timeout,
            "b2" => "a2",
            "13 77 88 99 aa bb cc dd" => "a3",
            "02 ee ff" => "02 cc dd",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff" => "cc dd");
        trx!(x, "aa bb" => "cc dd");
    }

    // B.3 Error handling B.3.4 Chaining - Scenario 19, 20
    #[test_log::test(tokio::test)]
    async fn test_error_picc_chaining_1() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 12 34" => "12 00 11 22 33 44 55 66",
            "a3" => timeout,
            "a3" => "13 77 88 99 aa bb cc dd",
            "a2" => "02 ee ff",
            "03 aa bb" => "03 cc dd",
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "12 34" => "00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff");
        trx!(x, "aa bb" => "cc dd");
    }

    #[test_log::test(tokio::test)]
    async fn test_error_retries() {
        let mock = mock!(
            "e0 80" => "06 77 77 81 02 80",
            "02 12 34" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
            "b2" => timeout,
        );
        let x = &mut IsoDepA::new(mock).await.unwrap();
        x.fsc = 10;
        trx!(x, "12 34" => Error::Communication);
    }
}
