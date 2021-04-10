#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m::asm::delay;
use stm32f4::stm32f401 as device;
use stm32f4::stm32f401::TIM1;
use stm32f4_playground as _; // Global logger + panicking-behavior

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("RGB LED PWM! PB15 = Red, PB14 = Green, PB13 = Blue.");

    // Take ownership of the device peripherals singleton
    if let Some(dp) = device::Peripherals::take() {
        // Take and own RCC RegisterBlock out of dp
        let rcc = dp.RCC;
        // Take and own the GPIOB and TIM1 struct out of dp
        let (gpiob, tim1) = (dp.GPIOB, dp.TIM1);

        /* GPIO configuration: PB15 = Red, PB14 = Green, PB13 = Blue */
        // Enable clock for GPIOB
        rcc.ahb1enr.write(|w| w.gpioben().enabled());
        // Set mode of PB13-15 as alternative function
        gpiob.moder.modify(|_, w| w.moder13().alternate());
        gpiob.moder.modify(|_, w| w.moder14().alternate());
        gpiob.moder.modify(|_, w| w.moder15().alternate());

        // Alternate function mapping 1 for TIM1_CHxN (see DS9716 datasheet)
        gpiob.afrh.modify(|_, w| w.afrh13().af1()); // TIM1_CH1N
        gpiob.afrh.modify(|_, w| w.afrh14().af1()); // TIM1_CH2N
        gpiob.afrh.modify(|_, w| w.afrh15().af1()); // TIM1_CH3N

        // Set GPIO speed for PB13-15 as high speed
        gpiob.ospeedr.modify(|_, w| w.ospeedr13().high_speed());
        gpiob.ospeedr.modify(|_, w| w.ospeedr14().high_speed());
        gpiob.ospeedr.modify(|_, w| w.ospeedr15().high_speed());
        // Set PB13-15 as push-pull
        gpiob.otyper.modify(|_, w| w.ot13().push_pull());
        gpiob.otyper.modify(|_, w| w.ot14().push_pull());
        gpiob.otyper.modify(|_, w| w.ot15().push_pull());

        /* TIM1 PWM configuration */
        rcc.apb2enr.write(|w| w.tim1en().enabled());
        // Setup TIM1 in PWM mode
        tim1.setup_as_pwm();
        // Turn everything off (assuming common cathode)
        tim1.set_rgb(0, 0, 0);

        loop {
            tim1.set_rgb(255, 0, 0); // Red
            delay(10_000_000); // Delay for at least n instruction cycles
            tim1.set_rgb(0, 255, 0); // Red
            delay(10_000_000); // Delay for at least n instruction cycles
            tim1.set_rgb(0, 0, 255); // Green
            delay(10_000_000); // Delay for at least n instruction cycles
        }
    };

    defmt::panic!("Uh oh, reached unreachable code!");
}

pub trait TIM1Ext {
    fn setup_as_pwm(&self);
    fn set_red(&self, value: u16);
    fn set_green(&self, value: u16);
    fn set_blue(&self, value: u16);
    fn set_rgb(&self, r: u16, g: u16, b: u16);
}

/// Writes to the Capture/Compare register (ccrX where X is the channel)
/// which in turns determines the duty cycle of each PWM pin.
/// Each channel corresponds to a specific physical pin which is connected
/// to a red, green, or blue LED pin.
impl TIM1Ext for TIM1 {
    /// Setup up the timer and channels in PWM mode, this assumes your main
    /// program has already enabled the clock for TIM1, via the RCC register
    fn setup_as_pwm(&self) {
        // Up-counting
        self.cr1.modify(|_, w| w.dir().up());
        // Clock prescalar (16 bit value, max 65,535)
        self.psc.write(|w| w.psc().bits(2500 - 1));
        // Auto-realod value, for up counting goes from 0->ARR
        self.arr.write(|w| w.arr().bits(255 - 1));
        // PWM Mode 1 output on channel 1, 2, 3
        // Output channel 1, 2, 3 preload enabled
        self.ccmr1_output().write(|w| {
            w.oc1m()
                .pwm_mode1()
                .oc1pe()
                .enabled()
                .oc2m()
                .pwm_mode1()
                .oc2pe()
                .enabled()
        });
        self.ccmr2_output()
            .write(|w| w.oc3m().pwm_mode1().oc3pe().enabled());
        // Enable complementary output of channel 1, 2, 3
        self.ccer.write(|w| {
            w.cc1ne()
                .set_bit()
                .cc1p()
                .clear_bit()
                .cc2ne()
                .set_bit()
                .cc2p()
                .clear_bit()
                .cc3ne()
                .set_bit()
                .cc3p()
                .clear_bit()
        });
        // Main output enable
        self.bdtr.write(|w| w.moe().enabled());
        // Enable counter
        self.cr1.modify(|_, w| w.cen().enabled());
    }

    fn set_red(&self, value: u16) {
        self.ccr3.write(|w| w.ccr().bits(value));
    }

    fn set_green(&self, value: u16) {
        self.ccr2.write(|w| w.ccr().bits(value));
    }

    fn set_blue(&self, value: u16) {
        self.ccr1.write(|w| w.ccr().bits(value));
    }

    fn set_rgb(&self, r: u16, g: u16, b: u16) {
        self.ccr3.write(|w| w.ccr().bits(r));
        self.ccr2.write(|w| w.ccr().bits(g));
        self.ccr1.write(|w| w.ccr().bits(b));
    }
}
