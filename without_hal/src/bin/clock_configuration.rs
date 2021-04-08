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
    defmt::info!("Gotta go fast!");

    // Take ownership of the core & device peripheral singletons
    if let (Some(cp), Some(dp)) = (
        cortex_m::Peripherals::take(),
        device::Peripherals::take(),
    ) {
        cortex_m::interrupt::free(move |cs| {
            // Take and own SYST (systick) out of cp
            let mut systick = cp.SYST;
            // Take and own FLASH & RCC RegisterBlock out of dp
            let (flash, rcc) = (dp.FLASH, dp.RCC);
            // Take and own the GPIOC struct out of dp
            let gpioc = dp.GPIOC;

            // Initialize clock to use PLL and HSI for 84 MHz frequency
            system_clock_init(&flash, &rcc);

            // Use internal clock provided by the core for SysTick
            systick.set_clock_source(SystClkSource::Core);
            // Reload value must be less than 0x00FFFFFF
            // With N = 840_0000, light toggles every 100 ms at f_clk = 84 MHz
            systick.set_reload(840_0000 - 1);
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

/// SystemCoreClock = ((INPUT_CLK / PLL_M) * PLL_N) / PLL_P
/// See Section 6,  Figure 12 of RM0368
fn system_clock_init(flash: &device::FLASH, rcc: &device::RCC) {
    // TODO: Replace this with safe code after PAC update
    // https://github.com/stm32-rs/stm32-rs/pull/374
    // To read data from FLASH memory, the correct number of wait states must
    // be set, two wait states, if 60 < HCLK ≤ 84 and 2.7V - 3.6V
    #[allow(unsafe_code)]
    unsafe {
        flash.acr.write(|w| w.bits(2));
    }

    // Enable the Internal High Speed oscillator (HSI)
    rcc.cr.modify(|_, w| w.hsion().on());
    while rcc.cr.read().hsirdy().is_not_ready() {}

    // Select HSI as clock source for PLL
    rcc.pllcfgr.modify(|_, w| w.pllsrc().hsi());
    // Configure PLL to output 84 MHz, where HSI = 16 MHz
    // ((16 / 16) * 336) / 4 = 84
    #[allow(unsafe_code)]
    unsafe {
        rcc.pllcfgr.modify(|_, w| {
            w.pllm()
                .bits(16)
                .plln()
                .bits(336)
                .pllp()
                .div4()
                .pllq()
                .bits(7)
        });
    }

    // Enable Phase Lock Loop (PLL)
    rcc.cr.modify(|_, w| w.pllon().on());
    while rcc.cr.read().pllrdy().is_not_ready() {}

    // AHB will run at 84 MHz, APB1 at 42 MHz, APB2 at 84 MHz
    rcc.cfgr
        .modify(|_, w| w.hpre().div1().ppre1().div2().ppre2().div1());

    // Select PLL as system clock input
    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}
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
