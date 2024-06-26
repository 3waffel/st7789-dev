#![allow(dead_code)]

use chrono::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};
use sysinfo::{System, SystemExt};
use textwrap::wrap;

use crate::data::*;
use crate::types::*;

#[derive(Debug)]
pub enum LayoutType {
    Home,
    Menu,
    SystemInfo,
    Wifi,
}

pub struct LayoutManager<'a> {
    layout_area: Rectangle,
    text_style: MonoTextStyle<'a, Rgb565>,
    current_layout: LayoutType,
    system: System,
}

impl LayoutManager<'_> {
    pub fn new() -> Self {
        Self {
            layout_area: Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
            text_style: MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
            current_layout: LayoutType::Home,
            system: System::new_all(),
        }
    }

    pub fn draw(&mut self, display: &mut SpiDisplay) {
        self.create_header(display);
        self.create_footer(display);

        match self.current_layout {
            LayoutType::Home => self.create_home_layout(display),
            LayoutType::Menu => self.create_menu_layout(display),
            LayoutType::SystemInfo => self.create_system_info_layout(display),
            LayoutType::Wifi => self.create_wifi_layout(display),
        }
    }

    pub fn input(&mut self, key: PinMap) {
        let layout = &mut self.current_layout;
        match layout {
            LayoutType::Home => match key {
                PinMap::KeyOk => *layout = LayoutType::Menu,
                PinMap::KeyCancel => *layout = LayoutType::SystemInfo,
                _ => {}
            },
            LayoutType::Menu => match key {
                PinMap::KeyMain => *layout = LayoutType::Home,
                _ => {}
            },
            LayoutType::SystemInfo => match key {
                PinMap::KeyMain => *layout = LayoutType::Home,
                _ => {}
            },
            LayoutType::Wifi => match key {
                PinMap::KeyMain => *layout = LayoutType::Home,
                _ => {}
            },
        }
    }

    pub fn create_label(&self, position: Point, display: &mut SpiDisplay) {}

    pub fn create_select_list(&self, position: Point, display: &mut SpiDisplay) {}

    pub fn create_header(&self, display: &mut SpiDisplay) {
        let uptime = self.system.uptime();
        let version = self.system.long_os_version().unwrap();
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

    pub fn create_footer(&self, display: &mut SpiDisplay) {
        let mut left: String = "null".into();
        let mut middle: String = "home".into();
        let mut right: String = "null".into();
        match self.current_layout {
            LayoutType::Home => {
                left = "menu".into();
                middle = "null".into();
                right = "system-info".into();
            }
            LayoutType::Menu => {}
            LayoutType::SystemInfo => {}
            LayoutType::Wifi => {}
        }
        let footer_text = format!("{left} | {middle} | {right}");

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

    pub fn create_menu_layout(&self, display: &mut SpiDisplay) {
        let char_h = self.text_style.font.character_size.height;
        let width = self.layout_area.size.width;
        let height = self.layout_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, char_h as i32 + 10), Size::new(width, height));

        let content = "Menu Screen".to_string();
        draw_text(
            &area,
            &content,
            self.text_style,
            Rgb565::CSS_DARK_SLATE_GRAY,
            display,
        );
    }

    pub fn create_system_info_layout(&mut self, display: &mut SpiDisplay) {
        let char_h = self.text_style.font.character_size.height;
        let width = self.layout_area.size.width;
        let height = self.layout_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, char_h as i32 + 10), Size::new(width, height));

        let info = get_system_info(&mut self.system);
        draw_list(
            &area,
            &info.iter().collect(),
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
