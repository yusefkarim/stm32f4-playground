#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::char;
use stm32f4::stm32f401 as device;
use stm32f4_playground as _; // Global logger + panicking-behavior

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Configuring USART1 to 9600 baud. PB6 is TX and PB7 is RX!");

    // Take ownership of the device peripherals singleton
    if let Some(dp) = device::Peripherals::take() {
        let rcc = dp.RCC;
        let gpiob = dp.GPIOB;
        let usart1 = dp.USART1;

        /* GPIO configuration: PB6 = USART1_TX, PB7 = USART1_RX */
        // Enable clock for GPIOB
        rcc.ahb1enr.write(|w| w.gpioben().enabled());
        // Set mode of PB6-7 as alternative function
        gpiob
            .moder
            .write(|w| w.moder6().alternate().moder7().alternate());
        // Alternate function mapping 7 for USART1 (see DS9716 datasheet)
        gpiob.afrl.write(|w| w.afrl6().af7().afrl7().af7());
        // Set GPIO speed for PB6-7 as high speed
        gpiob
            .ospeedr
            .write(|w| w.ospeedr6().high_speed().ospeedr7().high_speed());
        // Set PB6-7 as pull-up
        gpiob
            .pupdr
            .write(|w| w.pupdr6().pull_up().pupdr7().pull_up());
        // Set PB6-7 as push-pull
        gpiob
            .otyper
            .write(|w| w.ot6().push_pull().ot7().push_pull());

        /* USART1 configuration */
        // Enable clock for USART1
        rcc.apb2enr.modify(|_, w| w.usart1en().enabled());
        // Disable USART1 while we configure it
        usart1.cr1.modify(|_, w| w.ue().disabled());
        // Set data length to  8 bits
        usart1.cr1.modify(|_, w| w.m().m8());
        // Select 1 stop bit
        usart1.cr2.modify(|_, w| w.stop().stop1());
        // Set parity control as no parity
        usart1.cr1.modify(|_, w| w.pce().disabled());
        // Oversampling by 16, means OVER8 = 0
        usart1.cr1.modify(|_, w| w.over8().oversample16());
        // Set baudrate of 9600, assuming 16 MHz clock
        // USARTDIV = f_clk / (8 * (2 - OVER8) * baudrate)
        //          = 16 MHz / (8 * (2 - 0) * 9600)
        //          = 104.17
        //  DIV_Fraction = 16*0.17 = 2.72 ~= 0x3
        //  DIV_Mantissa = 104.17 ~= 0x68
        usart1
            .brr
            .modify(|_, w| w.div_mantissa().bits(0x68).div_fraction().bits(0x3));
        // Enable transmission and reception
        usart1.cr1.modify(|_, w| w.re().enabled().te().enabled());
        // Enable USART1
        usart1.cr1.modify(|_, w| w.ue().enabled());

        // NOTE: This is very inefficient. Only for learning purposes.
        let mut read_byte: u16;
        loop {
            // Wait until hardware sets RXNE bit
            while !usart1.sr.read().rxne().bit_is_set() {}
            // Reading from DR clears RXNE flag
            read_byte = usart1.dr.read().dr().bits();
            // Wait until hardware sets TXE bit
            while !usart1.sr.read().txe().bit_is_set() {}
            // Write the received character back as uppercase (if applicable)
            if let Some(c) = char::from_u32(read_byte as u32) {
                // Writing to DR clears TXE bit
                usart1
                    .dr
                    .write(|w| w.dr().bits(c.to_ascii_uppercase() as u16));
            }
            // Wait until TC = 1
            while usart1.sr.read().tc().bit_is_clear() {}
        }
    };

    defmt::panic!();
}
