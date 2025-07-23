# Wokwi Buzzer

## Build

```
cargo build
elf2uf2-rs target/thumbv6m-none-eabi/debug/wokwi-buzzer-rs
```

## Setup

VSCodeでWokwi Simulatorをインストール

必要なツールをインストール
```
cargo install probe-rs-tools
cargo install flip-link
cargo install elf2uf2-rs
rustup target add thumbv6m-none-eabi
```

参考：https://github.com/sksat/pico-wokwi-playground