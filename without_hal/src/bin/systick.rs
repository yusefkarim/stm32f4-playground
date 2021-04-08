#![deny(unsafe_code)]
#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use core::{cell::RefCell, ops::Deref};
use cortex_m::{asm::wfi, interrupt::Mutex, peripheral::syst::SystClkSource};
use stm32f4::stm32f401 as device;

static GPIOC: Mutex<RefCell<Option<device::GPIOC>>> = Mutex::new(RefCell::new(None));

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Starting blinky via SysTick interrupts!");

    // Take ownership of the core & device peripheral singletons
    if let (Some(cp), Some(dp)) = (
        cortex_m::Peripherals::take(),
        device::Peripherals::take(),
    ) {
        cortex_m::interrupt::free(move |cs| {
            // Take and own SYST (systick) out of cp
            let mut systick = cp.SYST;
            // Take and own RCC RegisterBlock out of dp
            let rcc = dp.RCC;
            // Take and own the GPIOC struct out of dp
            let gpioc = dp.GPIOC;

            // Use internal clock provided by the core for SysTick
            // NOTE: See `clock_configuration.rs` for how to configure the clock frequency
            systick.set_clock_source(SystClkSource::Core);
            // Reload value must be less than 0x00FFFFFF
            systick.set_reload(1_440_000 - 1);
            systick.clear_current();

            // Enable GPIOC clock
            rcc.ahb1enr.write(|w| w.gpiocen().enabled());
            // Set PC13 as an output
            gpioc.moder.write(|w| w.moder13().output());
            // Set PC13 to low speed (default)
            gpioc.ospeedr.write(|w| w.ospeedr13().low_speed());
            // Set PC13 as no pull-up, no pull-down
            gpioc.pupdr.write(|w| w.pupdr13().floating());

            // Transfer GPIOC into shared global structure
            GPIOC.borrow(cs).replace(Some(gpioc));

            // Enable SysTick counter & interrupt
            systick.enable_counter();
            systick.enable_interrupt();
        });
    };

    loop {
        wfi();
    }
}

// This is the exception handler that gets called when the the SysTick
// triggers an exception after its countdown
#[cortex_m_rt::exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref gpioc) = GPIOC.borrow(cs).borrow().deref() {
            if gpioc.odr.read().odr13().is_high() {
                gpioc.odr.write(|w| w.odr13().low()); // ON
            } else {
                gpioc.odr.write(|w| w.odr13().high()); // OFF
            }
        }
    });
}
