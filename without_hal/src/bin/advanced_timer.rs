#![deny(unsafe_code)]
#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use cortex_m::asm::wfi;
use stm32f4::stm32f401 as device;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Hook up an LED to PA7, TIM1 is toggling it!");

    // Take ownership of the device peripheral singleton
    if let Some(dp) = device::Peripherals::take() {
        cortex_m::interrupt::free(move |_| {
            // Take and own RCC, SYSCFG, and EXTI RegisterBlocks out of dp
            let rcc = dp.RCC;
            // Take and own the GPIOA struct dp
            let gpioa = dp.GPIOA;
            // Take and own the TIM1 struct out of dp
            let tim1 = dp.TIM1;

            // Enable GPIOA clock
            rcc.ahb1enr.write(|w| w.gpioaen().enabled());

            // Set PA1 as alternate function
            gpioa.moder.write(|w| w.moder7().alternate());
            // Set alternate function 1 (01) for PA7 (TIM1_CH1N)
            gpioa.afrl.write(|w| w.afrl7().af1());
            // Set output speed of PA7 as low by clearing speed bits
            gpioa.ospeedr.write(|w| w.ospeedr7().low_speed());
            // Set PA7 as no pull-up, no pull-down
            gpioa.pupdr.write(|w| w.pupdr7().floating());

            // Setup and enable TIM1 CH1
            tim1_ch1_init(&rcc, &tim1);
        });
    };

    loop {
        wfi();
    }
}

/// Setup Channel 1 of the Advanced Timer TIM1 to output toggle
/// For this specific program f_clk = 16 MHz, so with the prescalar below,
/// f_counter = 16 MHz / 8000 = 2 KHz, then with the auto-reload value below,
/// counting period = 1000 / f_counter = 0.5 ms
/// Therefore, the output should toggle on/off every 0.5 ms
fn tim1_ch1_init(rcc: &device::RCC, tim1: &device::TIM1) {
    // Enable advanced TIM1 clock
    rcc.apb2enr.modify(|_, w| w.tim1en().enabled());
    // Set count direction as up-counting
    tim1.cr1.write(|w| w.dir().up());
    // Clock prescalar (16 bit value, max 65,535)
    tim1.psc.write(|w| w.psc().bits(8000 - 1));
    // Auto-realod value, for up counting goes from 0->ARR
    tim1.arr.write(|w| w.arr().bits(1000 - 1));
    // Capture/compare register can be any value 0 < CCR < ARR
    tim1.ccr1.write(|w| w.ccr().bits(500));
    // Main output enable (MOE): 0 = Disable, 1 = Enable
    tim1.bdtr.write(|w| w.moe().enabled());
    // Select toggle mode (0011) for channel 1
    tim1.ccmr1_output().write(|w| w.oc1m().toggle());
    // Select output polarity as active high
    // Enable output for channel 1 complementary output
    tim1.ccer.write(|w| w.cc1np().clear_bit().cc1ne().set_bit());
    // Enable TIM1 counter
    tim1.cr1.write(|w| w.cen().enabled());
}
