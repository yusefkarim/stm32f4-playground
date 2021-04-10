#![deny(unsafe_code)]
#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use cortex_m::asm::delay;
use stm32f4::stm32f401 as device;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Starting blinky!");

    // Take ownership of the device peripherals singleton
    if let Some(dp) = device::Peripherals::take() {
        // Take and own RCC RegisterBlock out of dp
        let rcc = dp.RCC;
        // Take and own the GPIOC struct out of dp
        let gpioc = dp.GPIOC;

        // Enable GPIOC clock
        rcc.ahb1enr.write(|w| w.gpiocen().enabled());
        // Set PC13 as an output
        gpioc.moder.write(|w| w.moder13().output());
        // Set PC13 to low speed (default)
        gpioc.ospeedr.write(|w| w.ospeedr13().low_speed());
        // Set PC13 as no pull-up, no pull-down
        gpioc.pupdr.write(|w| w.pupdr13().floating());

        loop {
            gpioc.odr.write(|w| w.odr13().clear_bit()); // ON
            delay(5_000000); // Delay for at least n instruction cycles
            gpioc.odr.write(|w| w.odr13().set_bit()); // OFF
            delay(5_000000); // Delay for at least n instruction cycles
        }
    };

    defmt::panic!("Uh oh, reached unreachable code!");
}
