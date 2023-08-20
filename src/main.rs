use std::time::{Duration, Instant};

use anyhow::Result;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::Builder;
use rppal::gpio::{Gpio, OutputPin};
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use sysinfo::{ComponentExt, CpuExt, DiskExt, System, SystemExt};
use tracing::{debug, info};

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

    backlight.set_high();
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let char_h = 10;

    let mut sys = System::new_all();
    info!("Starting main loop");
    loop {
        display.clear(Rgb565::BLACK).unwrap();

        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_components();
        let mut text_y = char_h;
        let mut data: Vec<String> = vec![];
        data.append(
            &mut sys
                .cpus()
                .iter()
                .enumerate()
                .map(|(i, cpu)| format!("CPU{i}: {:.1}%", cpu.cpu_usage()))
                .collect::<Vec<_>>(),
        );
        data.push(format!(
            "MEM: {:.1}G/{:.1}G",
            sys.used_memory() as f32 / (1024 * 1024 * 1024) as f32,
            sys.total_memory() as f32 / (1024 * 1024 * 1024) as f32
        ));
        data.append(
            &mut sys
                .components()
                .iter()
                .map(|c| {
                    format!(
                        "{:3} {:.1}C/{:.1}C",
                        c.label(),
                        c.temperature(),
                        c.critical().unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>(),
        );
        data.append(
            &mut sys
                .disks()
                .iter()
                .map(|disk| {
                    format!(
                        "{:3} {:.1}G/{:.1}G",
                        disk.name().to_str().unwrap_or_default(),
                        disk.available_space() as f32 / (1024 * 1024 * 1024) as f32,
                        disk.total_space() as f32 / (1024 * 1024 * 1024) as f32
                    )
                })
                .collect::<Vec<_>>(),
        );
        for text in data.iter() {
            text_y += char_h;
            Text::new(&text, Point::new(0, text_y), text_style)
                .draw(&mut display)
                .unwrap();
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    backlight.set_low();
    display.clear(Rgb565::BLACK).unwrap();

    Ok(())
}
