#![no_std]
#![no_main]
#![deny(unsafe_code)]

use cortex_m::asm::delay;
use stm32f4_playground as _; // Global logger + panicking-behavior
use stm32f4xx_hal::{prelude::*, pwm, stm32 as device};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Plug your RGB LED into PA8-10! Don't forget current limiting resistors!");
    // Take ownership of the device peripherals singleton
    if let Some(dp) = device::Peripherals::take() {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split();
        let channels = (
            gpioa.pa8.into_alternate_af1(),  // Red
            gpioa.pa9.into_alternate_af1(),  // Green
            gpioa.pa10.into_alternate_af1(), // Blue
        );

        let pwm = pwm::tim1(dp.TIM1, channels, clocks, 20u32.khz());
        let (mut r, mut g, mut b) = pwm;
        let max_duty = r.get_max_duty();
        r.enable();
        g.enable();
        b.enable();

        loop {
            r.set_duty(max_duty);
            g.set_duty(0);
            b.set_duty(0);
            delay(5_000_000);
            r.set_duty(0);
            g.set_duty(max_duty);
            b.set_duty(0);
            delay(5_000_000);
            r.set_duty(0);
            g.set_duty(0);
            b.set_duty(max_duty);
            delay(5_000_000);
        }
    }

    defmt::panic!("Unreachable code");
}
