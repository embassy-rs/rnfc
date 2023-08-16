#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Must go FIRST
mod fmt;

use core::cell::RefCell;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
// global logger
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::rcc::{self};
use embassy_stm32::spi::{Config, Phase, Polarity, Spi};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Duration, Timer};
use rnfc::iso14443a::Poller;
use rnfc::iso_dep::IsoDepA;
use rnfc::traits::iso_dep::Reader;
use rnfc_st25r39::{SpiInterface, St25r39, WakeupConfig, WakeupMethodConfig, WakeupPeriod, WakeupReference};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = rcc::ClockSrc::PLL(
        rcc::PLLSource::HSI16,
        rcc::PLLClkDiv::Div2,
        rcc::PLLSrcDiv::Div1,
        rcc::PLLMul::Mul8,
        None,
    );
    let p = embassy_stm32::init(config);

    //let mut led = Output::new(p.PC4, Level::High, Speed::Low);

    let mut config = Config::default();
    config.mode.polarity = Polarity::IdleLow;
    config.mode.phase = Phase::CaptureOnSecondTransition;
    config.frequency = Hertz(1_000_000);
    let spi_bus = Spi::new(p.SPI1, p.PA5, p.PA7, p.PE14, NoDma, NoDma, config);
    let spi_bus = Mutex::<NoopRawMutex, _>::new(RefCell::new(spi_bus));
    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi_device = SpiDevice::new(&spi_bus, cs);
    let iface = SpiInterface::new(spi_device);
    let irq = ExtiInput::new(Input::new(p.PE15, Pull::None), p.EXTI15);
    let mut st = St25r39::new(iface, irq).await.unwrap();

    let config = WakeupConfig {
        period: WakeupPeriod::Ms500,
        capacitive: None,
        inductive_amplitude: None,
        inductive_phase: Some(WakeupMethodConfig {
            delta: 3,
            reference: WakeupReference::Automatic,
        }),
    };

    match st.wait_for_card(config).await {
        Ok(()) => {}
        Err(e) => warn!("wait for card failed: {:?}", e),
    }

    /*
    let conf = AatConfig {
        a_min: 0,
        a_max: 255,
        a_start: 128,
        a_step: 32,
        b_min: 0,
        b_max: 255,
        b_start: 128,
        b_step: 32,
        pha_target: 128,
        pha_weight: 2,
        amp_target: 196,
        amp_weight: 1,
    };
    st.mode_on().await;
    //st.iso14443a_start().await.unwrap();
    st.aat(conf).await;
    info!("DONE");
    return;
      */

    /*
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        let iso14 = st.start_iso14443a().await.unwrap();

        let mut poller = Poller::new(iso14);

        let card = match poller.select_any().await {
            Ok(x) => x,
            Err(e) => {
                warn!("Failed to select card: {:?}", e);
                continue;
            }
        };

        let mut card = IsoDepA::new(card).await.unwrap();
    }
       */

    'out: loop {
        Timer::after(Duration::from_millis(1000)).await;

        let iso14 = st.start_iso14443a().await.unwrap();

        let mut poller = Poller::new(iso14);
        let cards = poller.search::<8>().await.unwrap();
        info!("found cards: {:02x}", cards);

        for uid in cards {
            info!("checking card {:02x}", uid);

            let card = match poller.select_by_id(&uid).await {
                Ok(x) => x,
                Err(e) => {
                    warn!("Failed to select card with UID {:02x}: {:?}", uid, e);
                    continue;
                }
            };

            let mut card = match IsoDepA::new(card).await {
                Ok(x) => x,
                Err(e) => {
                    warn!("Failed ISO-DEP select, not an ISO-DEP card? {:?}", e);
                    continue;
                }
            };

            let mut rx = [0; 256];
            let tx = [0x90, 0x60, 0x00, 0x00, 0x00];

            match card.transceive(&tx, &mut rx).await {
                Ok(n) => info!("rxd: {:02x}", &rx[..n]),
                Err(e) => warn!("trx failed: {:?}", e),
            };

            match card.deselect().await {
                Ok(()) => {}
                Err(e) => {
                    warn!("deselect failed: {:?}", e);
                    continue 'out;
                }
            }
        }
    }
}
