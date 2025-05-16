use std::io::Error;

use hex_literal::hex;
use log::{trace, warn};
use nusb::transfer::RequestBuffer;
use nusb::Interface;
use rnfc_traits::iso_dep::Reader;

pub struct Device {
    iface: Interface,
}

pub struct Card<'a> {
    dev: &'a mut Device,

    pub atqa: [u8; 2],
    pub sak: u8,
    pub uid: Vec<u8>,
    pub ats: Vec<u8>,
}

impl<'a> Reader for Card<'a> {
    type Error = Error;
    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        let mut data = vec![0; 1 + tx.len()];
        data[0] = 0x01;
        data[1..].copy_from_slice(tx);

        let res = self.dev.pn53x_cmd(0x40, &data).await?;

        // check status byte
        if res[0] != 0x00 {
            return Err(Error::other(format!("transceive failed, status=0x{:02x}", res[0])));
        }

        rx[..res.len() - 1].copy_from_slice(&res[1..]);
        Ok(res.len() - 1)
    }
}

const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x82;

impl Device {
    pub async fn new() -> Result<Self, Error> {
        let di = nusb::list_devices()?
            .find(|d| d.vendor_id() == 0x072f && d.product_id() == 0x2200)
            .ok_or_else(|| Error::other("no USB device found"))?;
        trace!("Device info: {di:?}");

        let device = di.open().unwrap();
        if let Err(e) = device.reset() {
            warn!("acr122u reset failed: {}", e);
        }
        let iface = device.claim_interface(0)?;

        let mut this = Self { iface };

        this.init().await?;

        Ok(this)
    }

    async fn init(&mut self) -> Result<(), Error> {
        // turn on
        self.transfer(Vec::from(hex!("62000000000000010000"))).await?;

        // set PICC operating parameter: disable everything.
        self.transfer(Vec::from(hex!("6f050000000000000000 ff00510000"))).await?;

        // GetFirmwareVersion
        // This seems to "abort" previous command, retry it a few times?
        let mut res = Ok(());
        for _ in 0..3 {
            res = self.pn53x_cmd(0x02, &[]).await.map(|_| ());
            if res.is_ok() {
                break;
            }
        }
        res?;

        // SetParameters: Enable auto-RATS, auto-ATR_RES
        self.pn53x_cmd(0x12, &[0x14]).await?;

        self.pn53x_cmd(0x32, &hex!("02 000b0a")).await?;
        self.pn53x_cmd(0x32, &hex!("04 00")).await?;
        self.pn53x_cmd(0x32, &hex!("05 010001")).await?;

        // analog settings for Type A 106 kbps
        // Original settings are 59 F4 3F 11 4D 85 61 6F 26 62 87
        // but RxThreshold=0xF5 makes it work better on clone ACR122u's.
        self.pn53x_cmd(0x32, &hex!("0a 59 F4 3F 11 4D f5 61 6F 26 62 87")).await?;
        //                             |  |  |  |  |  |  |  |  |  |  TxBitPhase
        //                             |  |  |  |  |  |  |  |  |  MifNFC
        //                             |  |  |  |  |  |  |  |  ModWidth
        //                             |  |  |  |  |  |  |  GsNOff
        //                             |  |  |  |  |  |  Demod when own RF is Off
        //                             |  |  |  |  |  RxThreshold
        //                             |  |  |  |  Demod when own RF is On
        //                             |  |  |  ModGsP
        //                             |  |  CWGsP
        //                             |  GsNOn
        //                             RFCfg

        // analog settings for Type F 212/424 kbps
        self.pn53x_cmd(0x32, &hex!("0b 69 FF 3F 11 41 85 61 6F")).await?;

        // analog settings for Type B 106 kbps
        self.pn53x_cmd(0x32, &hex!("0c FF 04 85")).await?;

        // analog settings for 14443-4 212/424/848 kbps
        self.pn53x_cmd(0x32, &hex!("0d 85 15 8A 85 08 B2 85 01 DA")).await?;

        Ok(())
    }

    pub async fn beep(&mut self) -> Result<(), Error> {
        self.transfer(Vec::from(hex!("6f090000000000000000 ff0040ad0402000101")))
            .await?;
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<Card<'_>, Error> {
        let res = self.pn53x_cmd(0x4a, &[0x01, 0x00]).await?;

        if res[0] != 0x01 {
            return Err(Error::other("no card present"));
        }
        assert!(res[1] == 0x01);

        let atqa: [u8; 2] = res[2..4].try_into().unwrap();
        let sak = res[4];
        let uid_len = res[5] as usize;
        let uid = res[6..][..uid_len].to_vec();
        let ats_len = res[6 + uid_len] as usize;
        let ats = res[6 + uid_len..][..ats_len].to_vec();

        trace!("atqa: {:02x?}", atqa);
        trace!("sak: {:02x?}", sak);
        trace!("uid: {:02x?}", uid);
        trace!("ats: {:02x?}", ats);

        Ok(Card {
            dev: self,
            atqa,
            sak,
            uid,
            ats,
        })
    }

    async fn pn53x_cmd(&mut self, code: u8, data: &[u8]) -> Result<Vec<u8>, Error> {
        assert!(data.len() <= 255);

        let mut buf = vec![0; 10 + 5 + 2 + data.len()];
        // Build CCID header
        buf[0] = 0x6f;
        buf[1..5].copy_from_slice(&(5 + 2 + data.len() as u32).to_le_bytes());
        // Build fake APDU header
        buf[10] = 0xFF;
        buf[14] = (2 + data.len()) as u8;
        // Build PN53x header
        buf[15] = 0xd4;
        buf[16] = code;
        // data payload
        buf[17..].copy_from_slice(data);

        // do it!
        let res = self.transfer(buf).await?;

        // Strip CCID header
        if res.len() < 10 {
            return Err(Error::other(format!("unexpected CCID response length {}", res.len())));
        }
        if res[0] != 0x80 {
            return Err(Error::other(format!("unexpected CCID response 0x{:02x}", res[0])));
        };
        if res[0] != 0x00 && res[8] != 0x81 {
            return Err(Error::other(format!("CCID error 0x{:02x}", res[8])));
        };
        let len = u32::from_le_bytes(res[1..5].try_into().unwrap()) as usize;
        if res.len() < 10 + len {
            return Err(Error::other(format!("CCID resp too short: want {} got {}", len, res.len())));
        }
        let res = &res[10..][..len];

        // Strip fake APDU footer
        if res.len() < 2 {
            return Err(Error::other(format!("APDU resp too short: {}", res.len())));
        }
        let apdu_result = &res[res.len() - 2..];
        if apdu_result != &[0x90, 0x00] {
            return Err(Error::other(format!("APDU response code {:02x?}", apdu_result)));
        }
        let res = &res[..res.len() - 2];

        // Strip PN53x header.
        if res.len() < 2 {
            return Err(Error::other(format!("PN53x resp too short: {}", res.len())));
        }
        if res[0] != 0xd5 {
            return Err(Error::other(format!("PN53x unexpected resp 0x{:02x}", res[0])));
        }
        if res[1] != code + 1 {
            return Err(Error::other(format!(
                "PN53x unexpected resp cmd: want 0x{:02x}, got 0x{:02x}",
                code + 1,
                res[1],
            )));
        }

        Ok(res[2..].to_vec())
    }

    async fn transfer(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        trace!("tx: {:02x?}", data);
        self.iface.bulk_out(EP_OUT, data).await.into_result()?;
        let res = self.iface.bulk_in(EP_IN, RequestBuffer::new(256)).await.into_result()?;
        trace!("rx: {:02x?}", res);
        Ok(res)
    }
}
