# Learning embedded systems with an STM32F4 and Rust!

This repository is primarily for learning and trying random things.
You may find basic code for learning embedded at the register level in [without_hal](without_hal).
Eventually, there will be more interesting things using a hardware abstraction library (HAL) in [with_hal](with_hal).

## Development Board
- [STM32F401CC](https://www.st.com/en/microcontrollers-microprocessors/stm32f401cc.html)
- [WeAct STM32F4x1](https://github.com/WeActTC/MiniF4-STM32F4x1)
- [Banggood STM32F401 Development Board](https://www.banggood.com/STM32F401-Development-Board-STM32F401CCU6-STM32F4-Learning-Board-p-1568897.html?rmmds=search&cur_warehouse=CN)

## Getting started:
These are the high-level steps and requirements you need to run the examples in this repo.
Please see [app-template](https://github.com/knurling-rs/app-template) for more detail.

Install Cargo related tooling:
```sh
cargo install flip-link
cargo install probe-run
cargo install cargo-binutils              # Optional
rustup component add llvm-tools-preview   # Optional
```

Setup udev rules:
```sh
# 1. Create and edit new udev rule file
sudo vim /etc/udev/rules.d/70-st-link.rules

# 2. Add the following four lines
# STM32F3DISCOVERY rev A/B - ST-LINK/V2
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"
# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"

# 3. Reload and retrigger your rules
udevadm control --reload
udevadm trigger
```

Uploading code:
```sh
cargo run --bin ${TARGET}
# Or using shortcut (defined in .cargo/config.toml):
cargo rb ${TARGET}
```

## Resources:

* [cargo-binutils](https://github.com/rust-embedded/cargo-binutils)
* [cargo-make](https://github.com/sagiegurari/cargo-make)
* [OpenOCD](http://openocd.org/)
* [cargo-flash](https://github.com/probe-rs/cargo-flash)
* [cortex-m-rt startup code crate](https://docs.rs/cortex-m-rt/0.6.12/cortex_m_rt/)
* [cortex-m low-level access crate](https://docs.rs/cortex-m/0.6.2/cortex_m/)
* [stm32f4 peripheral access crate](https://docs.rs/crate/stm32f4/0.10.0)
* [The Embedded Rust Book](https://rust-embedded.github.io/book/)
* [Real Time For the Masses](https://github.com/rtfm-rs/cortex-m-rtfm)
* [A look into ways to implement and share data with interrupt handlers in Rust by therealprof](https://therealprof.github.io/blog/interrupt-comparison/)

## Miscellaneous Commands

```sh
# Flash a program with OpenOCD, replace ${TARGET_BIN} with your binary
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg -c "program ${TARGET_BIN} reset exit 0x08000000"
# Create a raw binary from an ELF, replace ${TARGET_ELF} with your compiled Rust code
# ${TARGET_BIN} can be named whatever you like
cargo objcopy --bin ${TARGET_ELF} -- -O binary ${TARGET_BIN}
# Use OpenOCD to erase all flash memory on target board
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg -c "init; reset halt; stm32f4x mass_erase 0; exit"
# Use semi-hosting to see debug output, requires STlink debugger
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg -c "init; arm semihosting enable"
# Attach to running OpenOCD server via GDB
arm-none-eabi-gdb -q ${TARGET_ELF} -ex "target remote localhost:3333"
```
