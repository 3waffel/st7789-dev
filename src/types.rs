use anyhow::Result;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
};
use mipidsi::{models::ST7789, Display};
use rppal::gpio::{Gpio, InputPin};
use rppal::{gpio::OutputPin, spi::Spi};

pub type SpiDisplay = Display<SPIInterfaceNoCS<Spi, OutputPin>, ST7789, OutputPin>;

pub struct DisplayWrapper(pub SpiDisplay);

impl Drop for DisplayWrapper {
    fn drop(&mut self) {
        self.0.clear(Rgb565::BLACK).unwrap();
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PinMap {
    SpiRst = 27,
    SpiDc = 25,
    Backlight = 24,
    KeyUp = 6,
    KeyDown = 19,
    KeyLeft = 5,
    KeyRight = 26,
    KeyPress = 13,
    KeyOk = 21,
    KeyMain = 20,
    KeyCancel = 16,
}

impl PinMap {
    pub fn get_input_pin(self) -> Result<InputPin> {
        let gpio = Gpio::new()?;
        Ok(gpio.get(self as u8)?.into_input_pullup())
    }
}

pub static KEY_TYPE: [PinMap; 8] = [
    PinMap::KeyUp,
    PinMap::KeyDown,
    PinMap::KeyLeft,
    PinMap::KeyRight,
    PinMap::KeyPress,
    PinMap::KeyOk,
    PinMap::KeyMain,
    PinMap::KeyCancel,
];
