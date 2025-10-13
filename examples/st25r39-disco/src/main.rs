#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::{info, warn};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::spi::{Config, Phase, Polarity, Spi};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use rnfc::iso_dep::IsoDepA;
use rnfc::iso14443a::Poller;
use rnfc::traits::iso_dep::Reader;
use rnfc_st25r39::{DriverResistance, SpiInterface, St25r39, WakeupConfig, WakeupMethodConfig, WakeupPeriod, WakeupReference};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL8,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2),
        });
    }
    let p = embassy_stm32::init(config);

    //let mut led = Output::new(p.PC4, Level::High, Speed::Low);

    let mut config = Config::default();
    config.mode.polarity = Polarity::IdleLow;
    config.mode.phase = Phase::CaptureOnSecondTransition;
    config.frequency = Hertz(1_000_000);
    let spi_bus = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PE14, config);
    let spi_bus = Mutex::<NoopRawMutex, _>::new(RefCell::new(spi_bus));
    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi_device = SpiDevice::new(&spi_bus, cs);
    let iface = SpiInterface::new(spi_device);
    let irq = ExtiInput::new(p.PE15, p.EXTI15, Pull::None);
    let mut st = St25r39::new(iface, irq).await.unwrap();

    let mut config = rnfc_st25r39::Config::new();
    config.driver_resistance = DriverResistance::Ohm1; // max power
    st.set_config(config);

    let wup_config = WakeupConfig {
        period: WakeupPeriod::Ms500,
        capacitive: None,
        inductive_amplitude: None,
        inductive_phase: Some(WakeupMethodConfig {
            delta: 3,
            reference: WakeupReference::Automatic,
        }),
    };

    match st.wait_for_card(wup_config).await {
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
