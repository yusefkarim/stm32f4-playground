#![no_std]
#![no_main]

use stm32f4_playground as _; // Global logger + panicking-behavior
use chacha20poly1305::aead::heapless::{consts::U128, Vec};
use chacha20poly1305::aead::{AeadInPlace, NewAead};
use chacha20poly1305::{ChaCha8Poly1305, Key, Nonce};
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;
use usbd_serial;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

/// Accepts a string via USB serial, encrypts the given string, then outputs the
/// encrypted string back out via the USB connection
/// Accepted strings should be terminated with ~ character
/// Ex. "Hello world~"
#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Unplug your debugger and send messages to be encrypted over USB!");
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr
        .use_hse(25.mhz())
        .sysclk(48.mhz())
        .require_pll48clk()
        .freeze();

    let gpioa = dp.GPIOA.split();
    let usb = USB {
        usb_global: dp.OTG_FS_GLOBAL,
        usb_device: dp.OTG_FS_DEVICE,
        usb_pwrclk: dp.OTG_FS_PWRCLK,
        pin_dm: gpioa.pa11.into_alternate_af10(),
        pin_dp: gpioa.pa12.into_alternate_af10(),
        hclk: clocks.hclk(),
    };

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut serial = usbd_serial::SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Yusef")
        .product("Crypto")
        .serial_number("1234")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    let key = Key::from_slice(b"an example very very secret key.");
    let cipher = ChaCha8Poly1305::new(key);
    let mut buffer: Vec<u8, U128> = Vec::new();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut tmp = [0u8; 64];

        match serial.read(&mut tmp) {
            Ok(count) if count > 0 => {
                if let Ok(_) = buffer.extend_from_slice(&tmp[0..count]) {
                    if buffer.ends_with(&[b'~']) {
                        let nonce = Nonce::from_slice(b"unique nonce");
                        if let Ok(_) = cipher.encrypt_in_place(nonce, b"", &mut buffer) {
                            serial.write(&buffer).unwrap();
                        } else {
                            serial.write(&"ENCRYPTION FAILED".as_bytes()).unwrap();
                        }
                        buffer.truncate(0);
                    };
                } else {
                    buffer.truncate(0);
                }
            }
            _ => {}
        }
    }
}
