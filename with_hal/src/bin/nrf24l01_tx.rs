#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use embedded_nrf24l01::{Configuration, CrcMode, DataRate, NRF24L01};
use hal::prelude::*;
use hal::spi::{Mode, Phase, Polarity, Spi};
use stm32f4xx_hal as hal;

macro_rules! success_blink {
    ($led: expr, $delay: expr) => {
        for _ in 0..10 {
            $led.set_high().unwrap();
            $delay.delay_ms(200_u32);
            $led.set_low().unwrap();
            $delay.delay_ms(200_u32);
        }
    };
}

#[cortex_m_rt::entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        hal::stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        let gpioc = dp.GPIOC.split();
        let gpioa = dp.GPIOA.split();

        // On-board LED
        let mut led = gpioc.pc13.into_push_pull_output();
        // Delay
        let mut delay = hal::delay::Delay::new(cp.SYST, clocks);
        // SPI Setup
        let sck = gpioa.pa5.into_alternate_af5();
        let miso = gpioa.pa6.into_alternate_af5();
        let mosi = gpioa.pa7.into_alternate_af5();
        let spi = Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            hal::time::KiloHertz(8000).into(),
            clocks,
        );
        // CE and CSN pins for nrf24l01
        let ce = gpioa.pa4.into_push_pull_output();
        let csn = gpioa.pa3.into_push_pull_output();
        // nrf24l01 setup
        let mut radio = NRF24L01::new(ce, csn, spi).unwrap();
        radio.set_frequency(8).unwrap();
        radio.set_auto_retransmit(0, 0).unwrap();
        radio.set_rf(&DataRate::R2Mbps, 3).unwrap();
        radio
            .set_pipes_rx_enable(&[true, false, false, false, false, false])
            .unwrap();
        radio.set_auto_ack(&[false; 6]).unwrap();
        radio.set_crc(CrcMode::Disabled).unwrap();
        radio.set_tx_addr(b"stm32").unwrap();
        radio.set_rx_addr(0, b"stm32").unwrap();
        let mut tx = radio.tx().unwrap();
        let data = b"hello";

        loop {
            led.set_high().unwrap(); // OFF
            delay.delay_ms(500_u32);
            if let Ok(_) = tx.wait_empty() {
                // Sending message
                led.set_low().unwrap(); // ON
                if let Ok(true) = tx.can_send() {
                    if tx.send(data).is_ok() {
                        success_blink!(led, delay);
                    }
                }
            }
            led.set_high().unwrap(); // OFF
            delay.delay_ms(500_u32);
        }
    }
    loop {}
}
