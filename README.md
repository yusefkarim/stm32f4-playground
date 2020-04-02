# Learning embedded systems with an STM32F4 and Rust!

## Development Board
- [STM32F401CC](https://www.st.com/en/microcontrollers-microprocessors/stm32f401cc.html)
- [WeAct STM32F4x1](https://github.com/WeActTC/MiniF4-STM32F4x1)
- [Banggood STM32F401 Development Board](https://www.banggood.com/STM32F401-Development-Board-STM32F401CCU6-STM32F4-Learning-Board-p-1568897.html?rmmds=search&cur_warehouse=CN)

## Language: [Rust](https://www.rust-lang.org/)

## Getting started:

TODO:
```sh
cargo install cargo-edit
cargo install cargo-flash
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

```sh
sudo vim /etc/udev/rules.d/70-st-link.rules
```

```udev
# STM32F3DISCOVERY rev A/B - ST-LINK/V2
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"

# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"
```

Uploading code:
```
cargo flash --release --chip STM32F401CCUx
```

## Goals:

1. Learn embedded systems by not using a HAL library
  - [] Learn basic GPIO
  - [] Learn about SYSTICK
  - [] Learn about Interrupts
  - [] Learn about Timers
  - [] Learn about UART
  - [] Learn about I2C
  - [] Learn about SPI
  - [] Learn about ADC/DAC
  - [] Learn about DMA

2. Make useful things via abstractions provided by HAL

3. Explore concurrency with RTFM

4. Explore concurreny using async/.await

5. Explore atomic instructions and when they should be used over other primitives

6. Contribute back to the Rust community

7. Explore Cortex-M4 specific use cases in DSP, cryptography, and distributed systems

## Resources:

1. [Real Time For the Masses](https://github.com/rtfm-rs/cortex-m-rtfm)

