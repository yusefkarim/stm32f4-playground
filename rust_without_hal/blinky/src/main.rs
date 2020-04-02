#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m::asm::wfi;
use stm32f4::stm32f401;


#[entry]
fn main() -> ! {
    loop {
        wfi();
    }
}
