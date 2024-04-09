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
    let dc = gpio.get(PinMap::SpiDc as u8)?.into_output();
    let rst = gpio.get(PinMap::SpiRst as u8)?.into_output();
    let mut backlight = gpio.get(PinMap::Backlight as u8)?.into_output();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 40_000_000_u32, Mode::Mode0)?;
    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut delay = Delay::new();
    let display = Builder::st7789(di)
        .with_display_size(WIDTH as u16, HEIGHT as u16)
        .with_orientation(mipidsi::Orientation::Landscape(true))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();
    let mut display = DisplayWrapper(display);

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

        let mut input = None;
        for key in KEY_TYPE {
            if key.get_input_pin()?.is_low() {
                input = Some(key);
                break;
            }
        }

        match input {
            Some(key) => {
                manager.input(key);
                manager.draw(&mut display.0);
                last_input_time = Instant::now();
                last_refresh_time = Instant::now();
            }
            None => {
                if timeout_duration > Duration::from_secs(20) {
                    display.0.clear(Rgb565::BLACK).unwrap();
                } else if refresh_interval > Duration::from_secs(3) {
                    last_refresh_time = Instant::now();
                    manager.draw(&mut display.0);
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
