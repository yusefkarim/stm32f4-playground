#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate panic_halt;
use cortex_m_rt::entry;
// use cortex_m::asm::wfi;


#[entry]
fn main() -> ! {
    loop {
        // wfi();
    }
}
