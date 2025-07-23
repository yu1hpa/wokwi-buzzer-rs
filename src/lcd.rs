//use crate::hal::Timer;
use hd44780_driver::bus::I2CBus;
use hd44780_driver::HD44780;
use rp_pico::hal::i2c::{ValidPinScl, ValidPinSda, I2C};
use rp_pico::hal::pac::I2C0;
use rp_pico::hal::Timer;

const LCD_ADDR: u8 = 0x27;

pub struct LcdDisplay<SDA, SCL>
where
    SDA: ValidPinSda<I2C0>,
    SCL: ValidPinScl<I2C0>,
{
    lcd: HD44780<I2CBus<I2C<I2C0, (SDA, SCL)>>>,
}

impl<SDA, SCL> LcdDisplay<SDA, SCL>
where
    SDA: ValidPinSda<I2C0>,
    SCL: ValidPinScl<I2C0>,
{
    pub fn new(i2c: I2C<I2C0, (SDA, SCL)>, mut timer: Timer) -> Self {
        let mut lcd = HD44780::new_i2c(i2c, LCD_ADDR, &mut timer)
            .unwrap_or_else(|_| core::panic!("LCD init error!"));
        lcd.clear(&mut timer).unwrap();
        Self { lcd }
    }

    pub fn write_line(&mut self, text: &str, timer: &mut Timer) {
        self.lcd.clear(timer).unwrap();
        self.lcd.write_str(text, timer).unwrap();
    }
}
