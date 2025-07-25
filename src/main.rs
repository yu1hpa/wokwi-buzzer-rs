#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::pac;
use hal::uart::*;
use heapless::String;
use panic_halt as _;
use rp_pico::hal;

use crate::lcd::LcdDisplay;
mod lcd;

const LCD_UPDATE_INTERVAL: u32 = 50000;

struct Status {
    lock: bool,
    display_value: u32,
}

impl Status {
    fn new() -> Self {
        Self {
            lock: false,
            display_value: 0u32,
        }
    }

    fn set_lock(&mut self) {
        self.lock = true;
    }

    fn unset_lock(&mut self) {
        self.lock = false;
    }

    fn inc_value(&mut self) {
        self.display_value += 1;
    }

    fn reset_value(&mut self) {
        self.display_value = 0;
    }
}

#[hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::timer::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // LCD Display
    let sda_pin = pins
        .gpio12
        .into_pull_up_input()
        .into_function::<hal::gpio::FunctionI2C>();
    let scl_pin = pins
        .gpio13
        .into_pull_up_input()
        .into_function::<hal::gpio::FunctionI2C>();

    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400_000u32.Hz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    let mut lcd = LcdDisplay::new(i2c, timer);

    // LED
    let mut rgb_led_red_pin = pins.gpio28.into_push_pull_output();
    let mut _rgb_led_green_pin = pins.gpio27.into_push_pull_output();
    let mut rgb_led_blue_pin = pins.gpio26.into_push_pull_output();

    // Button
    let mut red_button = pins.gpio18.into_pull_up_input();
    let mut blue_button = pins.gpio19.into_pull_up_input();
    let mut reset_button = pins.gpio20.into_pull_up_input();
    let mut correct_button = pins.gpio4.into_pull_up_input();
    let mut incorrect_button = pins.gpio5.into_pull_up_input();

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_function(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_function(),
    );
    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    writeln!(uart, "Hello, world!\r").unwrap();

    lcd.write_line("@yu1hpa", &mut timer).unwrap();
    timer.delay_ms(500);

    let mut loop_cnt = 0u32;
    let mut status = Status::new();
    loop {
        // LCDディスプレイに早押しボタンが押されるまでカウントを表示
        if loop_cnt % LCD_UPDATE_INTERVAL == 0 && !status.lock {
            let mut s: String<64> = String::new(); // heapless
            write!(&mut s, "{}", status.display_value).unwrap();
            lcd.write_line(&s, &mut timer).unwrap();

            status.inc_value();
        }

        // 赤色のボタン
        if red_button.is_low().unwrap() && !status.lock {
            status.set_lock();
            rgb_led_red_pin.set_high().unwrap();
            lcd.write_line("Red!!", &mut timer).unwrap();
        }

        // 青色のボタン
        if blue_button.is_low().unwrap() && !status.lock {
            status.set_lock();
            rgb_led_blue_pin.set_high().unwrap();
            lcd.write_line("Blue!!", &mut timer).unwrap();
        }

        // 黒（リセット）ボタン
        if reset_button.is_low().unwrap() {
            rgb_led_red_pin.set_low().unwrap();
            rgb_led_blue_pin.set_low().unwrap();
            status.unset_lock();
            status.reset_value();
        }

        // 赤または青色のボタンが押された後、正解だった場合に押す
        if correct_button.is_low().unwrap() && status.lock {
            lcd.write_line("Correct!", &mut timer).unwrap();
        }

        // 赤または青色のボタンが押された後、不正解だった場合に押す
        if incorrect_button.is_low().unwrap() && status.lock {
            lcd.write_line("Incorrect!", &mut timer).unwrap();
        }

        loop_cnt += 1;
    }
}
