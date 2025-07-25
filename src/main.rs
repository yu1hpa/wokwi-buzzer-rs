#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;

// Hardware Abstraction Layer
use rp_pico::hal;
// periheral access crate
use hal::pac;

use panic_halt as _;

// traits
use core::fmt::Write;
use embedded_hal::digital::{InputPin, OutputPin};
use hal::clocks::Clock;
use hal::fugit::RateExtU32;
use hal::uart::*;
use heapless::String;

use crate::lcd::LcdDisplay;

mod lcd;

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
    let mut led_pin = pins.gpio5.into_push_pull_output();
    let mut led_button = pins.gpio20.into_pull_up_input();

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

    timer.delay_ms(500);
    lcd.write_line("@yu1hpa", &mut timer);

    let mut value = 0u32;
    let mut loop_cnt = 0u32;
    let mut s: String<64> = String::new();
    loop {
        if loop_cnt % 50000 == 0 {
            s.clear();
            write!(&mut s, "{}", value).unwrap();
            lcd.write_line(&s, &mut timer);
            value += 1;
        }

        if led_button.is_low().unwrap() {
            led_pin.set_high().unwrap();
        } else {
            led_pin.set_low().unwrap();
        }

        loop_cnt += 1;
    }
}
