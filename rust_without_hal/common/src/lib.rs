#![deny(unsafe_code)]
#![no_std]
use stm32f4::stm32f401::{FLASH, RCC};

/// TODO: Add documentation
/// SystemCoreClock = ((INPUT_CLK / PLL_M) * PLL_N) / PLL_P
/// See Section 6,  Figure 12 of RM0368
pub fn system_clock_init(flash: &FLASH, rcc: &RCC) {
    // TODO: Replace this with safe code after PAC update
    // https://github.com/stm32-rs/stm32-rs/pull/374
    // To read data from FLASH memory, the correct number of wait states must
    // be set, two wait states, if 60 < HCLK ≤ 84 and V_in -> 2.7V - 3.6V
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

// TODO: Give option to choose HSE or HSI as input to PLL
/*
enum ClockType {...}
pub fn system_clock_init(flash: &FLASH, rcc: &RCC, clock: ClockType) {
*/
