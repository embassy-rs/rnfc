#![no_std]
#![no_main]

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_nrf::config::LfclkSource;
use embassy_nrf::gpio::{Flex, Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Duration;
use rnfc::iso_dep::IsoDepA;
use rnfc::iso14443a::Poller;
use rnfc_fm175xx::{Fm175xx, I2cInterface, WakeupConfig};
use rnfc_traits::iso_dep::Reader;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    //config.hfclk_source = HfclkSource::ExternalXtal;
    config.lfclk_source = LfclkSource::ExternalXtal;
    let p = embassy_nrf::init(config);

    let npd = p.P0_15;
    let mut scl = p.P0_20;
    let mut sda = p.P0_22;
    let irq = p.P0_24;

    let npd = Output::new(npd, Level::High, OutputDrive::Standard);
    let irq = Input::new(irq, Pull::None);

    {
        // Try to unstick the i2c bus if it's stuck.
        let mut scl = Flex::new(scl.reborrow());
        scl.set_high();
        scl.set_as_input_output(Pull::None, OutputDrive::HighDrive0Disconnect1);
        let mut sda = Flex::new(sda.reborrow());
        sda.set_high();
        sda.set_as_input_output(Pull::None, OutputDrive::HighDrive0Disconnect1);

        if sda.is_low() {
            warn!("SDA stuck low.")
        }
        if scl.is_low() {
            warn!("SCL stuck low.")
        }

        info!("wiggling SCL...");
        for _ in 0..12 {
            scl.set_low();
            cortex_m::asm::delay(64_000_000 / 100_000 / 2);
            scl.set_high();
            cortex_m::asm::delay(64_000_000 / 100_000 / 2);

            if scl.is_low() {
                warn!("SCL still low while clocking it.")
            }
        }

        if sda.is_low() {
            warn!("SDA still stuck low.")
        }
        if scl.is_low() {
            warn!("SCL still stuck low.")
        }

        info!("doing start+stop");
        cortex_m::asm::delay(64_000_000 / 100_000 / 2);
        sda.set_low();
        cortex_m::asm::delay(64_000_000 / 100_000 / 2);
        sda.set_high();
        cortex_m::asm::delay(64_000_000 / 100_000 / 2);

        if sda.is_low() {
            warn!("SDA STILL stuck low, wtf?")
        }
        if scl.is_low() {
            warn!("SCL STILL stuck low, wtf?")
        }
    }

    let mut config = twim::Config::default();
    config.frequency = twim::Frequency::K400;
    config.scl_high_drive = true;
    config.sda_high_drive = true;

    let mut buf = [0u8; 256];
    let twim = Twim::new(p.TWISPI0, Irqs, sda, scl, config, &mut buf);

    let iface = I2cInterface::new(twim, 0x28);
    let mut fm = Fm175xx::new(iface, npd, irq).await;

    let wup_config = WakeupConfig {
        sleep_time: 2,
        prepare_time: 13,
        measure_time: 11,
        threshold: 20,
        n_drive: 1,
        p_drive: 4,
        recalibrate_interval: Some(Duration::from_secs(20 * 60)), // 20min
    };

    loop {
        fm.wait_for_card(wup_config).await.unwrap();

        let poller = fm.start_iso14443a().await.unwrap();
        let mut poller = Poller::new(poller);

        /*
        for card in poller.search::<8>().await.unwrap() {
            info!("got card: {:02x}", card);
        }
         */

        match poller.select_any().await {
            Err(e) => warn!("poll failed: {:?}", e),
            Ok(card) => {
                let mut isodep = IsoDepA::new(card).await.unwrap();
                let mut rx = [0; 256];
                let tx = [0x90, 0x60, 0x00, 0x00, 0x00];
                match isodep.transceive(&tx, &mut rx).await {
                    Err(e) => warn!("transceive failed: {:?}", e),
                    Ok(n) => info!("rxd: {:02x}", &rx[..n]),
                }
            }
        };
    }
}
