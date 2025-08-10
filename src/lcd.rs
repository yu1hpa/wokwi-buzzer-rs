use hd44780_driver::{
    bus::I2CBus,
    charset::{CharsetUniversal, Fallback},
    memory_map::MemoryMap1602,
    setup::DisplayOptionsI2C,
    HD44780,
};
use rp_pico::hal::i2c::{ValidPinScl, ValidPinSda, I2C};
use rp_pico::hal::pac::I2C0;
use rp_pico::hal::Timer;

const LCD_ADDR: u8 = 0x27;

#[derive(Debug)]
pub enum LcdError {
    InitError,
    ClearError,
    WriteStrError,
}

pub struct LcdDisplay<SDA, SCL>
where
    SDA: ValidPinSda<I2C0>,
    SCL: ValidPinScl<I2C0>,
{
    lcd: HD44780<I2CBus<I2C<I2C0, (SDA, SCL)>>, MemoryMap1602, Fallback<CharsetUniversal, 32>>,
}

impl<SDA, SCL> LcdDisplay<SDA, SCL>
where
    SDA: ValidPinSda<I2C0>,
    SCL: ValidPinScl<I2C0>,
{
    pub fn new(i2c: I2C<I2C0, (SDA, SCL)>, mut timer: Timer) -> Result<Self, LcdError> {
        let options = DisplayOptionsI2C::new(MemoryMap1602::new()).with_i2c_bus(i2c, LCD_ADDR);
        let mut lcd = match HD44780::new(options, &mut timer) {
            Ok(display) => display,
            Err(_) => return Err(LcdError::InitError),
        };
        if let Err(_) = lcd.clear(&mut timer) {
            return Err(LcdError::ClearError);
        }
        Ok(Self { lcd })
    }

    pub fn write_line(&mut self, text: &str, timer: &mut Timer) -> Result<(), LcdError> {
        match self.lcd.clear(timer) {
            Ok(_) => (),
            Err(_err) => return Err(LcdError::ClearError),
        }

        match self.lcd.write_str(text, timer) {
            Ok(_) => (),
            Err(_err) => return Err(LcdError::WriteStrError),
        }

        Ok(())
    }
}
