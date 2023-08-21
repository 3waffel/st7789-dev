use std::time::Duration;

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
use rppal::gpio::{Gpio, OutputPin};
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use sysinfo::{ComponentExt, CpuExt, DiskExt, System, SystemExt};
use tracing::info;

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
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let mut sys = System::new_all();
    info!("Starting main loop");
    loop {
        // display.clear(Rgb565::BLACK).unwrap();

        let top_area = Rectangle::new(Point::new(0, 0), Size::new(240, 20));
        let uptime = sys.uptime();
        let header_text = format!(
            "{}:{:02}:{:02} {}",
            uptime / 3600,
            (uptime % 3600) / 60,
            (uptime % 60),
            sys.long_os_version().unwrap_or_default()
        );
        draw_data(
            &top_area,
            text_style,
            &vec![&header_text],
            &mut display,
            Rgb565::CSS_DARK_VIOLET,
        );

        let data = get_system_info(&mut sys);
        let left_area = Rectangle::new(Point::new(0, 20), Size::new(100, 100));
        let right_area = Rectangle::new(Point::new(100, 20), Size::new(140, 100));
        let bottom_area = Rectangle::new(Point::new(0, 120), Size::new(240, 120));
        draw_data(
            &left_area,
            text_style,
            &data[..6].iter().collect::<Vec<_>>(),
            &mut display,
            Rgb565::CSS_DARK_SLATE_GRAY,
        );
        draw_data(
            &right_area,
            text_style,
            &data[6..].iter().collect::<Vec<_>>(),
            &mut display,
            Rgb565::CSS_DARK_SLATE_GRAY,
        );
        draw_data(
            &bottom_area,
            text_style,
            &vec![],
            &mut display,
            Rgb565::CSS_DARK_SLATE_GRAY,
        );

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    backlight.set_low();
    display.clear(Rgb565::BLACK).unwrap();

    Ok(())
}

fn get_system_info(sys: &mut System) -> Vec<String> {
    sys.refresh_cpu();
    sys.refresh_memory();
    sys.refresh_components();
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
        sys.used_memory() as f32 / (1024_i32.pow(3)) as f32,
        sys.total_memory() as f32 / (1024_i32.pow(3)) as f32
    ));
    data.append(
        &mut sys
            .components()
            .iter()
            .map(|c| {
                format!(
                    "{:.3}: {:.1}C/{:.1}C",
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
                    disk.available_space() as f32 / (1024_i32.pow(3)) as f32,
                    disk.total_space() as f32 / (1024_i32.pow(3)) as f32
                )
            })
            .collect::<Vec<_>>(),
    );
    data
}

fn draw_data(
    area: &Rectangle,
    text_style: MonoTextStyle<'_, Rgb565>,
    data: &Vec<&String>,
    display: &mut Display<SPIInterfaceNoCS<Spi, OutputPin>, ST7789, OutputPin>,
    background_color: Rgb565,
) {
    let x = area.top_left.x;
    let y = area.top_left.y;
    let width = area.size.width;
    let height = area.size.height;
    let display = &mut display.clipped(&area);
    display.clear(background_color).unwrap();

    let char_h = text_style.font.character_size.height as i32;
    let char_w = text_style.font.character_size.width;
    let mut text_y = y;
    let count_per_line: usize = width as usize / char_w as usize;

    let mut split_data: Vec<&str> = vec![];
    data.iter().for_each(|line| {
        let mut remaining: &str = line;
        while remaining.len() > count_per_line {
            let split_index = remaining[..count_per_line]
                .rfind(char::is_whitespace)
                .unwrap_or(count_per_line);
            split_data.push(&remaining[..split_index]);
            remaining = &remaining[split_index..];
        }
        split_data.push(remaining);
    });
    for line in split_data.iter() {
        text_y += char_h;
        if text_y + char_h > y + height as i32 {
            break;
        }
        Text::new(&line, Point::new(x, text_y), text_style)
            .draw(display)
            .unwrap();
    }
}
