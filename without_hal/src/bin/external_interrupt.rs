#![deny(unsafe_code)]
#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use core::{cell::RefCell, ops::Deref};
use cortex_m::{asm::wfi, interrupt::Mutex, peripheral::NVIC};
use stm32f4::stm32f401 as device;
use stm32f4::stm32f401::interrupt;

static GPIOC: Mutex<RefCell<Option<device::GPIOC>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<device::EXTI>>> = Mutex::new(RefCell::new(None));

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("External interrupt enabled, trying pressing PA0!");

    // Take ownership of the device peripheral singleton
    if let Some(dp) = device::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            // Take and own RCC, SYSCFG, and EXTI RegisterBlocks out of dp
            let (rcc, syscfg, exti) = (dp.RCC, dp.SYSCFG, dp.EXTI);
            // Take and own the GPIO A & C structs out of dp
            let (gpioa, gpioc) = (dp.GPIOA, dp.GPIOC);

            // Enable GPIO A & C clocks
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
            // Configure EXTI0 to trigger on PA0 falling edge (button press)
            exti0_init(&rcc, &syscfg, &exti);

            // Transfer EXTI & GPIOC into shared global structures
            GPIOC.borrow(cs).replace(Some(gpioc));
            EXTI.borrow(cs).replace(Some(exti));
        });
    };

    // Enable EXTI0 interrupt
    #[allow(unsafe_code)]
    unsafe {
        NVIC::unmask(device::Interrupt::EXTI0);
    }

    loop {
        wfi();
    }
}

/// Configure EXTI0 and set the conditions for the interrupt to trigger
fn exti0_init(rcc: &device::RCC, syscfg: &device::SYSCFG, exti: &device::EXTI) {
    // Enabled system configuration controller clock
    rcc.apb2enr.modify(|_, w| w.syscfgen().enabled());

    // Set PA0 as the trigger source of EXTI0
    #[allow(unsafe_code)]
    unsafe {
        syscfg.exticr1.write(|w| w.exti0().bits(0));
    }

    // Disable EXTI0 rising edge trigger, via Rising trigger selection register
    exti.rtsr.modify(|_, w| w.tr0().disabled());
    // Enable EXTI0 falling edge trigger, via Falling trigger selection register
    exti.ftsr.modify(|_, w| w.tr0().enabled());
    // Unmask EXTI0 interrupt bit, allowing it to be enabled
    exti.imr.modify(|_, w| w.mr0().unmasked());
}

/// This is the interrupt handler that gets called when something triggers
/// the EXTI0 line
#[interrupt]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        // Toggle GPIOC
        if let Some(ref gpioc) = GPIOC.borrow(cs).borrow().deref() {
            if gpioc.odr.read().odr13().is_high() {
                gpioc.odr.write(|w| w.odr13().low()); // ON
            } else {
                gpioc.odr.write(|w| w.odr13().high()); // OFF
            }
        }
        // Clear interrupt pending request on EXTI0
        if let Some(exti) = EXTI.borrow(cs).borrow().deref() {
            exti.pr.write(|w| w.pr0().clear());
        }
    });
}
