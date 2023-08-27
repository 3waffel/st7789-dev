#![allow(dead_code)]

use anyhow::Result;
use chrono::prelude::*;
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
use sysinfo::{System, SystemExt};
use textwrap::wrap;
use tokio::sync::Mutex;
use tracing::info;

use crate::data::*;

#[derive(Debug)]
pub enum LayoutType {
    Home,
    Menu,
    SystemInfo,
    Wifi,
}

#[derive(Debug)]
pub enum KeyType {
    Up,
    Down,
    Left,
    Right,
    Press,
    Ok,
    Main,
    Cancel,
}

pub type SpiDisplay = Display<SPIInterfaceNoCS<Spi, OutputPin>, ST7789, OutputPin>;

pub struct LayoutManager<'a> {
    layout_area: Rectangle,
    text_style: MonoTextStyle<'a, Rgb565>,
    current_layout: Arc<Mutex<LayoutType>>,
    system: Arc<Mutex<System>>,
}

impl LayoutManager<'_> {
    pub fn new() -> Self {
        Self {
            layout_area: Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
            text_style: MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
            current_layout: Arc::new(Mutex::new(LayoutType::Home)),
            system: Arc::new(Mutex::new(System::new_all())),
        }
    }

    pub async fn draw(&self, display: &mut SpiDisplay) {
        self.create_header(display).await;
        self.create_footer(display).await;

        let lock = self.current_layout.clone();
        let layout = lock.lock().await;
        match *layout {
            LayoutType::Home => self.create_home_layout(display),
            LayoutType::Menu => self.create_menu_layout(display),
            LayoutType::SystemInfo => self.create_system_info_layout(display).await,
            LayoutType::Wifi => self.create_wifi_layout(display),
        }
    }

    pub async fn input(&self, key: KeyType) {
        let lock = self.current_layout.clone();
        let mut layout = lock.lock().await;
        match *layout {
            LayoutType::Home => match key {
                KeyType::Ok => *layout = LayoutType::Menu,
                KeyType::Cancel => *layout = LayoutType::SystemInfo,
                _ => {}
            },
            LayoutType::Menu => match key {
                KeyType::Ok => {}
                KeyType::Cancel => *layout = LayoutType::Home,
                _ => {}
            },
            LayoutType::SystemInfo => match key {
                KeyType::Cancel => *layout = LayoutType::Home,
                _ => {}
            },
            LayoutType::Wifi => match key {
                KeyType::Cancel => *layout = LayoutType::Home,
                _ => {}
            },
        }
    }

    pub fn create_label(&self, position: Point, display: &mut SpiDisplay) {}

    pub fn create_select_list(&self, position: Point, display: &mut SpiDisplay) {}

    pub async fn create_header(&self, display: &mut SpiDisplay) {
        let lock = self.system.clone();
        let sys = lock.lock().await;
        let uptime = sys.uptime();
        let version = sys.long_os_version().unwrap();
        let header_text = format!(
            "{}:{:02}:{:02} {}",
            uptime / 3600,
            (uptime % 3600) / 60,
            (uptime % 60),
            version
        );

        let char_h = self.text_style.font.character_size.height;
        let width = self.layout_area.size.width;
        let height = char_h + 10;
        let area = Rectangle::new(Point::new(0, 0), Size::new(width, height));

        draw_text(
            &area,
            &header_text,
            self.text_style,
            Rgb565::CSS_DARK_VIOLET,
            display,
        );
    }

    pub async fn create_footer(&self, display: &mut SpiDisplay) {
        let lock = self.current_layout.clone();
        let layout = lock.lock().await;
        let left: String;
        let middle: String = "null".into();
        let right: String;
        match *layout {
            LayoutType::Home => {
                left = "menu".into();
                right = "system-info".into();
            }
            LayoutType::Menu => {
                left = "null".into();
                right = "home".into();
            }
            LayoutType::SystemInfo => {
                left = "null".into();
                right = "home".into();
            }
            LayoutType::Wifi => {
                left = "null".into();
                right = "home".into();
            }
        }
        let footer_text = format!("{left}  {middle}  {right}");

        let char_h = self.text_style.font.character_size.height;
        let width = self.layout_area.size.width;
        let height = char_h + 10;
        let area = Rectangle::new(
            Point::new(0, (self.layout_area.size.height - char_h - 10) as i32),
            Size::new(width, height),
        );

        draw_text(
            &area,
            &footer_text,
            self.text_style,
            Rgb565::CSS_BLACK,
            display,
        );
    }

    pub fn create_home_layout(&self, display: &mut SpiDisplay) {
        let char_h = self.text_style.font.character_size.height;
        let width = self.layout_area.size.width;
        let height = self.layout_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, char_h as i32 + 10), Size::new(width, height));

        let dt = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        draw_text(
            &area,
            &dt,
            self.text_style,
            Rgb565::CSS_DARK_SLATE_GRAY,
            display,
        );
    }

    pub fn create_menu_layout(&self, display: &mut SpiDisplay) {}

    pub async fn create_system_info_layout(&self, display: &mut SpiDisplay) {
        let char_h = self.text_style.font.character_size.height;
        let width = self.layout_area.size.width;
        let height = self.layout_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, char_h as i32 + 10), Size::new(width, height));

        let lock = self.system.clone();
        let mut sys = lock.lock().await;
        draw_list(
            &area,
            &get_system_info(&mut sys).iter().collect(),
            self.text_style,
            Rgb565::CSS_DARK_SLATE_GRAY,
            display,
        );
    }

    pub fn create_wifi_layout(&self, display: &mut SpiDisplay) {}
}

pub fn draw_text(
    area: &Rectangle,
    text: &String,
    text_style: MonoTextStyle<'_, Rgb565>,
    background_color: Rgb565,
    display: &mut SpiDisplay,
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

    for line in wrap(text, count_per_line) {
        text_y += char_h;
        if text_y > y + height as i32 {
            break;
        }
        Text::new(&line, Point::new(x, text_y), text_style)
            .draw(display)
            .unwrap();
    }
}

pub fn draw_list(
    area: &Rectangle,
    list: &Vec<&String>,
    text_style: MonoTextStyle<'_, Rgb565>,
    background_color: Rgb565,
    display: &mut SpiDisplay,
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

    let mut split_data = vec![];
    list.iter().for_each(|line| {
        split_data.push(wrap(line, count_per_line));
    });
    for line in split_data.iter().flatten() {
        text_y += char_h;
        if text_y + char_h > y + height as i32 {
            break;
        }
        Text::new(&line, Point::new(x, text_y), text_style)
            .draw(display)
            .unwrap();
    }
}
