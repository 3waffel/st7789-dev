#![allow(dead_code)]

use cstr_core::CString;
use display_interface_spi::SPIInterfaceNoCS;
use embassy_executor::Executor;
use embassy_time as _;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::prelude::*;
use log::*;
use lvgl::{
    style::Style, widgets::Label, Align, Color, Display, DrawBuffer, Part, Screen, TextAlign,
    Widget,
};
use mipidsi::{
    options::{ColorInversion, Orientation},
    Builder,
};
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use static_cell::StaticCell;

pub mod data;
pub mod layout;
pub mod types;
use data::*;
use types::*;
// use layout::*;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let gpio = Gpio::new().unwrap();
    let dc = gpio.get(PinMap::SpiDc as u8).unwrap().into_output();
    let rst = gpio.get(PinMap::SpiRst as u8).unwrap().into_output();
    let mut backlight = gpio.get(PinMap::Backlight as u8).unwrap().into_output();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 40_000_000_u32, Mode::Mode0).unwrap();
    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut delay = Delay::new();
    let mut spi_display = Builder::st7789(di)
        .with_display_size(WIDTH as u16, HEIGHT as u16)
        .with_orientation(Orientation::Landscape(true))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();

    info!("SPI set up");

    backlight.set_pwm_frequency(100., 0.005).unwrap();
    backlight.set_high();
    lvgl::init();
    let buffer = DrawBuffer::<{ (WIDTH * HEIGHT) as usize }>::default();
    let display = Display::register(buffer, WIDTH, HEIGHT, |refresh| {
        spi_display.draw_iter(refresh.as_pixels()).unwrap();
    })
    .unwrap();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run(display)).unwrap();
    });
}

#[embassy_executor::task]
async fn run(display: Display) {
    let mut home_scr = display.get_scr_act().unwrap();
    let mut home_scr_style = Style::default();
    home_scr_style.set_bg_color(Color::from_rgb((0, 0, 0)));
    home_scr_style.set_radius(0);
    home_scr.add_style(Part::Main, &mut home_scr_style);

    let mut header_label = Label::new().unwrap();
    let mut header_style = Style::default();
    header_style.set_text_color(Color::from_rgb((255, 255, 255)));
    header_style.set_text_align(TextAlign::Center);
    header_label.add_style(Part::Main, &mut header_style);
    header_label.set_align(Align::TopMid, 0, 0);
    let val = CString::new(get_header_info()).unwrap();
    header_label.set_text(&val).unwrap();

    let mut info_label = Label::new().unwrap();
    let mut info_style = Style::default();
    info_style.set_text_color(Color::from_rgb((255, 255, 255)));
    info_style.set_text_align(TextAlign::Left);
    info_label.add_style(Part::Main, &mut info_style);
    info_label.set_align(Align::Center, 0, 0);
    info_label.set_width(WIDTH);
    let val = CString::new(get_system_info().join("\n")).unwrap();
    info_label.set_text(&val).unwrap();

    let mut blank_scr = Screen::blank().unwrap();
    let mut blank_scr_style = Style::default();
    blank_scr_style.set_bg_color(Color::from_rgb((0, 0, 0)));
    blank_scr_style.set_radius(0);
    blank_scr.add_style(Part::Main, &mut blank_scr_style);

    // let mut manager = LayoutManager::new(&mut display);
    let mut last_refresh_time = Instant::now();
    let mut last_input_time = Instant::now();

    info!("Starting main loop");

    loop {
        let current_time = Instant::now();
        let refresh_interval = current_time.duration_since(last_refresh_time);
        let timeout_duration = current_time.duration_since(last_input_time);

        lvgl::task_handler();
        let mut input = None;
        for key in KEY_TYPE {
            if key.get_input_pin().unwrap().is_low() {
                input = Some(key);
                break;
            }
        }

        match input {
            Some(key) => {
                // manager.input(key);
                // manager.draw();
                display.set_scr_act(&mut home_scr);
                last_input_time = Instant::now();
                last_refresh_time = Instant::now();
            }
            None => {
                if timeout_duration > Duration::from_secs(20) {
                    // spi_display.clear(Rgb565::BLACK).unwrap();
                    display.set_scr_act(&mut blank_scr);
                } else if refresh_interval > Duration::from_secs(3) {
                    last_refresh_time = Instant::now();
                    // manager.draw();
                    let val = CString::new(get_header_info()).unwrap();
                    header_label.set_text(&val).unwrap();
                    let val = CString::new(get_system_info().join("\n")).unwrap();
                    info_label.set_text(&val).unwrap();
                    display.set_scr_act(&mut home_scr);
                }
            }
        }
        Timer::after_millis(200).await;
        lvgl::tick_inc(Instant::now().duration_since(current_time).into());
    }
}
