#![no_std]
#![no_main]
/// ADXL345 via I2C
/// Make sure the CS and SDO pins are tied high
///
/// Dear reader: This is horrible code. Solely for learning purposes. It is not supposed to
/// represent a proper driver in any way.
///
/// TODO: Add some error checking like: https://github.com/stm32-rs/stm32f4xx-hal/blob/master/src/i2c.rs#L804
/// TODO: Look into type state pattern to ensure enable_clock() clock is called before moving on
use cortex_m::asm::delay;
use stm32f4::stm32f401 as device;
use stm32f4_playground as _; // Global logger + panicking-behavior

const ADXL345_ADDRESS: u8 = 0x53;
#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum ADXL345_Reg {
    DEVID = 0x0,        // Device ID
    DATAX0 = 0x32,      // X-axis data 0 (read 6 bytes for X/Y/Z)
    POWER_CTL = 0x2D,   // Power-saving features control
    DATA_FORMAT = 0x31, // Controls the presentation of data
    BW_RATE = 0x2c,
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("TODO!");

    // Take ownership of the device peripherals singleton
    if let Some(dp) = device::Peripherals::take() {
        // Take and own RCC RegisterBlock out of dp
        let rcc = dp.RCC;
        // Take and own GPIOB & I2C1 out of dp
        let (gpiob, i2c1) = (dp.GPIOB, dp.I2C1);

        /* GPIO configuration: PB6 = SCL1, PB7 = SDA1 */
        // Enable clock for GPIOB
        rcc.ahb1enr.write(|w| w.gpioben().enabled());
        // Set mode of PB6-7 as alternative function
        gpiob
            .moder
            .write(|w| w.moder6().alternate().moder7().alternate());
        // Alternate function mapping 4 for I2C1 (see DS9716 datasheet)
        gpiob.afrl.write(|w| w.afrl6().af4().afrl7().af4());
        // Set GPIO speed for PB6-7 as high speed
        gpiob
            .ospeedr
            .write(|w| w.ospeedr6().high_speed().ospeedr7().high_speed());
        // Set PB6-7 as open drain
        gpiob
            .otyper
            .write(|w| w.ot6().open_drain().ot7().open_drain());

        /* I2C1 setup */
        i2c1.enable_clock(&rcc);
        i2c1.init();
        defmt::info!("I2C Initialization Complete");
        let mut data = [0; 6];
        // i2c1.write(ADXL345_ADDRESS, &[ADXL345_Reg::DEVID as u8]);
        // i2c1.read(ADXL345_ADDRESS, &mut data[..1]);
        // defmt::info!("Device ID: {}", data[0]);
        // Put the device into measurement mode by setting the Measure Bit in POWER_CTL
        i2c1.write(ADXL345_ADDRESS, &[ADXL345_Reg::POWER_CTL as u8, 0x8]);
        // Set D0 = 1, D1 = 1 to get range of +- 16 g
        i2c1.write(ADXL345_ADDRESS, &[ADXL345_Reg::DATA_FORMAT as u8, 0x3]);

        loop {
            i2c1.write(ADXL345_ADDRESS, &[ADXL345_Reg::DATAX0 as u8]);
            i2c1.read(ADXL345_ADDRESS, &mut data);
            defmt::info!("X: {:?}", format(&data[0..2]));
            defmt::info!("Y: {:?}", format(&data[2..4]));
            defmt::info!("Z: {:?}", format(&data[4..6]));
            delay(5_000_000); // Delay for at least n instruction cycles
        }
    };

    defmt::panic!("Uh oh, reached unreachable code!");
}

/// Returns normalized acceleration value in m/s^2 (e.g., 1.0 == 9.8g)
fn format(val: &[u8]) -> f32 {
    let value = ((val[1] as i16) << 8) | val[0] as i16;
    // let value = value as i16;
    (value as f32 * ((16 * 2) as f32 / 1024.0)) as f32
}

trait I2CExt {
    fn enable_clock(&self, rcc: &device::RCC);
    fn init(&self);
    fn write(&self, addr: u8, bytes: &[u8]);
    fn read(&self, addr: u8, bytes: &mut [u8]);
}

impl I2CExt for device::I2C1 {
    fn enable_clock(&self, rcc: &device::RCC) {
        rcc.apb1enr.modify(|_, w| w.i2c1en().enabled());
        // Stall the pipeline to work around erratum 2.1.13 (DM00037591)
        cortex_m::asm::dsb();
    }

    fn init(&self) {
        // Disable I2C so we can configure it
        self.cr1.modify(|_, w| w.pe().disabled());
        // I2C mode
        self.cr1.modify(|_, w| w.smbus().i2c());
        // Enable clock stretching
        self.cr1.modify(|_, w| w.nostretch().enabled());
        // Enable analog noise filter, disable digital noise filter
        self.fltr.write(|w| w.anoff().enabled().dnf().no_filter());
        // 16 MHz frequency, assume default 16 MHz HSI being used for APB1
        let freq: u8 = 16;
        self.cr2.write(|w| unsafe { w.freq().bits(freq) });
        // Configure correct maximum rise time
        let trise: u32 = (freq as u32 * 300) / 1000 + 1;
        self.trise.write(|w| w.trise().bits(trise as u8));
        // Configure as Fm (fast) mode, max 400 kHz SCL clock frequency
        self.ccr.modify(|_, w| w.f_s().fast());
        // Fm mode 2:1 duty cycle, meaning DUTY = 0
        self.ccr.modify(|_, w| w.duty().duty2_1());
        // Let's use a 400 kHz SCL frequency (see RM0368 p.503):
        // f_SCL ~= 1 / (T_high + T_low)
        // where T_high = CCR*T_PCLK, T_low = 2*CCR*T_PCLK, and T_PCLK = 1/f_PCLK
        // Then,  CCR = f_PCLK / (3 * f_SCL) = 16 MHz / (3 * 400 kHz) ~= 13
        self.ccr.modify(|_, w| unsafe { w.ccr().bits(13) });
        // Enable I2C
        self.cr1.modify(|_, w| w.pe().enabled());
    }

    fn write(&self, addr: u8, bytes: &[u8]) {
        // Send a START condition
        self.cr1.modify(|_, w| w.start().start());
        // Wait until the START condition is generated
        while self.sr1.read().sb().is_no_start() {}
        // Wait until back in Master mode (MSL = 1) and communication in progress (BUSY = 1)
        while {
            let sr2 = self.sr2.read();
            sr2.msl().bit_is_clear() && sr2.busy().bit_is_clear()
        } {}
        // Send slave address
        self.dr.write(|w| unsafe { w.bits(u32::from(addr) << 1) });
        // Wait until address is sent
        while self.sr1.read().addr().bit_is_clear() {}
        // Clear ADDR condition by reading SR2
        self.sr2.read();
        for c in bytes {
            // Wait until DR is empty
            while self.sr1.read().tx_e().is_not_empty() {}
            // Write a byte
            self.dr.write(|w| unsafe { w.bits(u32::from(*c)) });
            // Wait until byte has been transferred
            while self.sr1.read().btf().is_not_finished() {}
        }
        // Send a STOP condition
        self.cr1.modify(|_, w| w.stop().stop());
        // Wait for STOP condition to transmit
        while self.cr1.read().stop().is_no_stop() {}
    }

    fn read(&self, addr: u8, bytes: &mut [u8]) {
        if let Some((last, bytes)) = bytes.split_last_mut() {
            // Send a START condition
            self.cr1.modify(|_, w| w.start().start());
            // Set ACK bit
            self.cr1.modify(|_, w| w.ack().ack());
            // Wait until the START condition is generated
            while self.sr1.read().sb().is_no_start() {}
            // Wait until back in Master mode (MSL = 1) and communication in progress (BUSY = 1)
            while {
                let sr2 = self.sr2.read();
                sr2.msl().bit_is_clear() && sr2.busy().bit_is_clear()
            } {}
            // Send slave address
            self.dr
                .write(|w| unsafe { w.bits((u32::from(addr) << 1) + 1) });
            // Wait until address is sent
            while self.sr1.read().addr().bit_is_clear() {}
            // Clear ADDR condition by reading SR2
            self.sr2.read();
            for c in bytes {
                // Wait until DR is not empty
                while self.sr1.read().rx_ne().is_empty() {}
                // Receive a byte
                *c = self.dr.read().bits() as u8;
            }
            // Set NACK bit
            self.cr1.modify(|_, w| w.ack().nak());
            // Send a STOP condition to stop receiving
            self.cr1.modify(|_, w| w.stop().stop());
            // Read in last byte
            while self.sr1.read().rx_ne().is_empty() {}
            *last = self.dr.read().bits() as u8;
            // Wait for STOP condition to transmit
            while self.cr1.read().stop().is_no_stop() {}
        }
    }
}
