use embassy_time::{Duration, Timer};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

use crate::{Error, Interface, St25r39};

pub struct AatConfig {
    pub a_min: u8,
    pub a_max: u8,
    pub a_start: u8,
    pub a_step: u8,
    pub b_min: u8,
    pub b_max: u8,
    pub b_start: u8,
    pub b_step: u8,
    pub pha_target: u8,
    pub pha_weight: u8,
    pub amp_target: u8,
    pub amp_weight: u8,
}

impl<I: Interface, IrqPin: InputPin + Wait> St25r39<I, IrqPin> {
    pub async fn aat(&mut self, conf: AatConfig) -> Result<(), Error<I::Error>> {
        let mut a = conf.a_start;
        let mut b = conf.b_start;
        let mut cost = self.aat_measure(a, b, &conf).await?;

        loop {
            let mut best = None;
            for (a, b) in [
                (a.saturating_add(conf.a_step).min(conf.a_max), b),
                (a.saturating_sub(conf.a_step).max(conf.a_min), b),
                (a, b.saturating_add(conf.b_step).min(conf.b_max)),
                (a, b.saturating_sub(conf.b_step).max(conf.b_min)),
            ] {
                let cost = self.aat_measure(a, b, &conf).await?;

                let new_best = match best {
                    None => true,
                    Some((old_cost, _, _)) => cost < old_cost,
                };

                if new_best {
                    best = Some((cost, a, b));
                }
            }

            let (new_cost, new_a, new_b) = best.unwrap();
            if new_cost >= cost {
                break;
            }

            cost = new_cost;
            a = new_a;
            b = new_b;
        }

        #[cfg(feature = "st25r3916")]
        self.regs().ant_tune_a().write_value(a)?;
        #[cfg(feature = "st25r3916")]
        self.regs().ant_tune_a().write_value(b)?;

        // st25r3911b chip might offer a better approach, simlar to what lib does
        // self.regs().ant_tune_ctrl().write(|v| v.set_tre(1))  ect
        // and whole procedure might need to be adjusted,
        // to take "CalibrateAntenna" command into account, ant_tune_target ect
        #[cfg(feature = "st25r3911b")]
        self.regs().ant_tune_ctrl().write_value(a.into())?;
        #[cfg(feature = "st25r3911b")]
        self.regs().ant_tune_ctrl().write_value(b.into())?;

        Ok(())
    }

    async fn aat_measure(&mut self, a: u8, b: u8, conf: &AatConfig) -> Result<u32, Error<I::Error>> {
        #[cfg(feature = "st25r3916")]
        self.regs().ant_tune_a().write_value(a)?;
        #[cfg(feature = "st25r3916")]
        self.regs().ant_tune_a().write_value(b)?;

        // Wait for caps to settle.
        Timer::after(Duration::from_millis(1)).await;

        info!("aa");
        let amp = self.measure_phase().await?;
        info!("aabb");
        let pha = 0u8;
        //let pha = self.measure_phase().await;
        info!("aabbcc");

        // calculate cost function
        let cost = amp.abs_diff(conf.amp_target) as u32 * conf.amp_weight as u32
            + pha.abs_diff(conf.pha_target) as u32 * conf.pha_weight as u32;

        info!("a={} b={} amp={} pha={} cost={}", a, b, amp, pha, cost);
        Ok(cost)
    }
}
