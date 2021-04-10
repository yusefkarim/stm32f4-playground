#![deny(unsafe_code)]
#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use stm32f4::stm32f401 as device;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Try pressing PA0 (on-board button)!");

    // Take ownership of the device peripherals singleton
    if let Some(dp) = device::Peripherals::take() {
        // Take and own RCC RegisterBlock out of dp
        let rcc = dp.RCC;
        // Take and own the GPIO A & C structs out of dp
        let (gpioa, gpioc) = (dp.GPIOA, dp.GPIOC);

        // Enable GPIO A & C clock
        rcc.ahb1enr
            .write(|w| w.gpiocen().enabled().gpioaen().enabled());

        // Set PC13 as an output
        gpioc.moder.write(|w| w.moder13().output());
        // Set PC13 to low speed (default)
        gpioc.ospeedr.write(|w| w.ospeedr13().low_speed());
        // Set PC13 as no pull-up, no pull-down
        gpioc.pupdr.write(|w| w.pupdr13().floating());

        // Set PA0 as an input
        gpioa.moder.write(|w| w.moder0().input());
        // Set PA0 as pull-up (normally high)
        gpioa.pupdr.write(|w| w.pupdr0().pull_up());

        // Turn PC13 off. NOTE: Reverse logic, high == OFF, low == ON
        gpioc.odr.write(|w| w.odr13().set_bit()); // OFF
        loop {
            // NOTE: This is not very reliable, you must consider button debouncing

            // If button is pressed
            if gpioa.idr.read().idr0().is_low() {
                // Turn PC13 ON (low) if it is currently OFF (high),
                // else turn PC13 OFF (high)
                if gpioc.idr.read().idr13().is_high() {
                    gpioc.odr.write(|w| w.odr13().clear_bit()); // ON
                } else {
                    gpioc.odr.write(|w| w.odr13().set_bit()); // OFF
                }
            }
        }
    };

    defmt::panic!("Uh oh, reached unreachable code!");
}
