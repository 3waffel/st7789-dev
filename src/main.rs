use anyhow::Result;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};
use mipidsi::{models::ST7789, Builder, Display};
use rppal::{
    gpio::{Gpio, OutputPin},
    hal::Delay,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{ComponentExt, CpuExt, DiskExt, System, SystemExt};
use tracing::info;

pub mod data;
pub mod layout;
use data::*;
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

    let layout_manager = Arc::new(LayoutManager::new());
    let input_handler = layout_manager.clone();
    let draw_handler = layout_manager.clone();
    tokio::spawn(async move {
        loop {
            if key_ok.is_low() {
                input_handler.input(KeyType::Ok).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            if key_cancel.is_low() {
                input_handler.input(KeyType::Cancel).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    loop {
        draw_handler.draw(&mut display).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    backlight.set_low();
    display.clear(Rgb565::BLACK).unwrap();

    Ok(())
}
