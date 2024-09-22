#![allow(dead_code)]

use chrono::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, iso_8859_2::FONT_10X20, MonoTextStyle},
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
pub enum ScreenOptions {
    Home,
    Menu,
    SystemInfo,
}

pub struct ThemeSchema {
    body_text_style: MonoTextStyle<'static, Rgb565>,
    headline_text_style: MonoTextStyle<'static, Rgb565>,
    header_background_color: Rgb565,
    body_background_color: Rgb565,
    footer_background_color: Rgb565,
}

impl ThemeSchema {
    pub fn new() -> Self {
        Self {
            body_text_style: MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
            headline_text_style: MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
            header_background_color: Rgb565::CSS_DARK_VIOLET,
            body_background_color: Rgb565::CSS_DARK_SLATE_GRAY,
            footer_background_color: Rgb565::BLACK,
        }
    }
}

pub struct LayoutManager {
    screen_area: Rectangle,
    theme: ThemeSchema,
    current_screen: ScreenOptions,
    system: System,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            screen_area: Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
            theme: ThemeSchema::new(),
            current_screen: ScreenOptions::Home,
            system: System::new_all(),
        }
    }

    pub fn draw(&mut self, display: &mut SpiDisplay) {
        self.create_header(display);
        self.create_footer(display);

        match self.current_screen {
            ScreenOptions::Home => self.create_home_layout(display),
            ScreenOptions::Menu => self.create_menu_layout(display),
            ScreenOptions::SystemInfo => self.create_system_info_layout(display),
        }
    }

    pub fn input(&mut self, key: PinMap) {
        let layout = &mut self.current_screen;
        match layout {
            ScreenOptions::Home => match key {
                PinMap::KeyOk => *layout = ScreenOptions::Menu,
                PinMap::KeyCancel => *layout = ScreenOptions::SystemInfo,
                _ => {}
            },
            ScreenOptions::Menu => match key {
                PinMap::KeyMain => *layout = ScreenOptions::Home,
                _ => {}
            },
            ScreenOptions::SystemInfo => match key {
                PinMap::KeyMain => *layout = ScreenOptions::Home,
                _ => {}
            },
        }
    }

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

        let text_style = self.theme.body_text_style;
        let char_h = text_style.font.character_size.height;
        let width = self.screen_area.size.width;
        let height = char_h + 10;
        let area = Rectangle::new(Point::new(0, 0), Size::new(width, height));

        draw_text(
            &area,
            &header_text,
            text_style,
            self.theme.header_background_color,
            display,
        );
    }

    pub fn create_footer(&self, display: &mut SpiDisplay) {
        let mut left: String = "null".into();
        let mut middle: String = "home".into();
        let mut right: String = "null".into();
        match self.current_screen {
            ScreenOptions::Home => {
                left = "menu".into();
                middle = "null".into();
                right = "system-info".into();
            }
            ScreenOptions::Menu => {}
            ScreenOptions::SystemInfo => {}
        }
        let footer_text = format!("{left} | {middle} | {right}");

        let text_style = self.theme.body_text_style;
        let char_h = text_style.font.character_size.height;
        let width = self.screen_area.size.width;
        let height = char_h + 10;
        let area = Rectangle::new(
            Point::new(0, (self.screen_area.size.height - height) as i32),
            Size::new(width, height),
        );

        draw_text(
            &area,
            &footer_text,
            text_style,
            self.theme.footer_background_color,
            display,
        );
    }

    pub fn create_home_layout(&self, display: &mut SpiDisplay) {
        let text_style = self.theme.headline_text_style;
        let char_h = text_style.font.character_size.height;
        let width = self.screen_area.size.width;
        let height = self.screen_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, 20), Size::new(width, height));

        let dt = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        draw_text(
            &area,
            &dt,
            text_style,
            self.theme.body_background_color,
            display,
        );
    }

    pub fn create_menu_layout(&self, display: &mut SpiDisplay) {
        let text_style = self.theme.headline_text_style;
        let char_h = text_style.font.character_size.height;
        let width = self.screen_area.size.width;
        let height = self.screen_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, 20), Size::new(width, height));

        let content = "Menu Screen".to_string();
        draw_text(
            &area,
            &content,
            text_style,
            self.theme.body_background_color,
            display,
        );
    }

    pub fn create_system_info_layout(&mut self, display: &mut SpiDisplay) {
        let text_style = self.theme.body_text_style;
        let char_h = text_style.font.character_size.height;
        let width = self.screen_area.size.width;
        let height = self.screen_area.size.height - 2 * char_h - 20;
        let area = Rectangle::new(Point::new(0, 20), Size::new(width, height));

        let info = get_system_info();
        draw_list(
            &area,
            &info.iter().collect(),
            text_style,
            self.theme.body_background_color,
            display,
        );
    }
}

/// draw wrapped text in target area
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
