use anyhow::Result;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use mipidsi::Builder;
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use std::time::Duration;
use std::time::Instant;
use tracing::info;

pub mod actor;
pub mod data;
pub mod layout;
pub mod types;
use layout::*;
use types::*;

const WIDTH: i32 = 240;
const HEIGHT: i32 = 240;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let gpio = Gpio::new()?;
    let dc = gpio.get(KeyMap::SpiDc as u8)?.into_output();
    let rst = gpio.get(KeyMap::SpiRst as u8)?.into_output();
    let mut backlight = gpio.get(KeyMap::Backlight as u8)?.into_output();

    let key_up = gpio.get(KeyMap::KeyUp as u8)?.into_input_pullup();
    let key_down = gpio.get(KeyMap::KeyDown as u8)?.into_input_pullup();
    let key_left = gpio.get(KeyMap::KeyLeft as u8)?.into_input_pullup();
    let key_right = gpio.get(KeyMap::KeyRight as u8)?.into_input_pullup();
    let key_press = gpio.get(KeyMap::KeyPress as u8)?.into_input_pullup();
    let key_ok = gpio.get(KeyMap::KeyOk as u8)?.into_input_pullup();
    let key_main = gpio.get(KeyMap::KeyMain as u8)?.into_input_pullup();
    let key_cancel = gpio.get(KeyMap::KeyCancel as u8)?.into_input_pullup();
    info!("GPIO set up");

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 40_000_000_u32, Mode::Mode0)?;
    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut delay = Delay::new();
    let mut display = Builder::st7789(di)
        .with_display_size(WIDTH as u16, HEIGHT as u16)
        .with_orientation(mipidsi::Orientation::Landscape(true))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();
    info!("SPI set up");

    backlight.set_pwm_frequency(100., 0.005)?;
    backlight.set_high();
    info!("Starting main loop");

    let mut manager = LayoutManager::new();
    let mut last_refresh_time = Instant::now();
    let mut last_input_time = Instant::now();
    loop {
        let current_time = Instant::now();
        let refresh_interval = current_time.duration_since(last_refresh_time);
        let timeout_duration = current_time.duration_since(last_input_time);

        let input = if key_ok.is_low() {
            Some(KeyMap::KeyOk)
        } else if key_main.is_low() {
            Some(KeyMap::KeyMain)
        } else if key_cancel.is_low() {
            Some(KeyMap::KeyCancel)
        } else {
            None
        };
        match input {
            Some(key) => {
                manager.input(key);
                manager.draw(&mut display);
                last_input_time = Instant::now();
                last_refresh_time = Instant::now();
            }
            None => {
                if timeout_duration > Duration::from_secs(20) {
                    display.clear(Rgb565::BLACK).unwrap();
                } else if refresh_interval > Duration::from_secs(3) {
                    last_refresh_time = Instant::now();
                    manager.draw(&mut display);
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    backlight.set_low();
    display.clear(Rgb565::BLACK).unwrap();

    Ok(())
}
