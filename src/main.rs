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
use layout::*;

const SPI_RST: u8 = 27;
const SPI_DC: u8 = 25;
const BACKLIGHT: u8 = 24;

const KEY_UP: u8 = 6;
const KEY_DOWN: u8 = 19;
const KEY_LEFT: u8 = 5;
const KEY_RIGHT: u8 = 26;
const KEY_PRESS: u8 = 13;
const KEY_OK: u8 = 21;
const KEY_MAIN: u8 = 20;
const KEY_CANCEL: u8 = 16;

const WIDTH: i32 = 240;
const HEIGHT: i32 = 240;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let gpio = Gpio::new()?;
    let dc = gpio.get(SPI_DC)?.into_output();
    let rst = gpio.get(SPI_RST)?.into_output();
    let mut backlight = gpio.get(BACKLIGHT)?.into_output();

    let key_up = gpio.get(KEY_UP)?.into_input_pullup();
    let key_down = gpio.get(KEY_DOWN)?.into_input_pullup();
    let key_left = gpio.get(KEY_LEFT)?.into_input_pullup();
    let key_right = gpio.get(KEY_RIGHT)?.into_input_pullup();
    let key_press = gpio.get(KEY_PRESS)?.into_input_pullup();
    let key_ok = gpio.get(KEY_OK)?.into_input_pullup();
    let key_main = gpio.get(KEY_MAIN)?.into_input_pullup();
    let key_cancel = gpio.get(KEY_CANCEL)?.into_input_pullup();
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

    backlight.set_pwm_frequency(100., 0.1)?;
    backlight.set_high();
    info!("Starting main loop");

    let mut manager = LayoutManager::new();
    let mut start_time = Instant::now();
    loop {
        let current_time = Instant::now();
        let duration = current_time.duration_since(start_time);

        let input = if key_ok.is_low() {
            Some(KeyType::Ok)
        } else if key_cancel.is_low() {
            Some(KeyType::Cancel)
        } else {
            None
        };
        match input {
            Some(key) => {
                manager.input(key);
                manager.draw(&mut display);
                start_time = Instant::now();
            }
            None => {
                if duration > Duration::from_secs(3) {
                    manager.draw(&mut display);
                    start_time = Instant::now();
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    backlight.set_low();
    display.clear(Rgb565::BLACK).unwrap();

    Ok(())
}
