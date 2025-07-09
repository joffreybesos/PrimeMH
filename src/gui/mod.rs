use notan::text::Font;

use crate::settings::Locales;

pub mod draw_buff_bar;
mod draw_item_log;
mod draw_item_tooltip;
mod draw_lines;
mod draw_map;
mod draw_objects;
mod draw_party_info;
mod draw_path;
mod draw_presets;
mod draw_units;
mod draw_world_units;
mod images;
pub mod ui;
mod util;
pub mod hotkeys;
pub mod egui;
pub mod play_sound;


pub struct Fonts {
    pub exocet_font: Font,
    pub formal_font: Font,
    pub korean_font: Font,
    pub taiwan_font: Font,
    pub blizzard_font: Font,
}

impl Fonts {
    pub fn get_safe_font(&self, locale: &Locales) -> &Font {
        match locale {
            Locales::enUS => &self.exocet_font,
            Locales::zhTW => &self.taiwan_font,
            Locales::deDE => &self.exocet_font,
            Locales::esES => &self.exocet_font,
            Locales::frFR => &self.exocet_font,
            Locales::itIT => &self.exocet_font,
            Locales::koKR => &self.korean_font,
            Locales::plPL => &self.exocet_font,
            Locales::enBG => &self.exocet_font,
            Locales::Unknown => &self.exocet_font,
        }
    }

    pub fn get_safe_font_formal(&self, locale: &Locales) -> &Font {
        match locale {
            Locales::enUS => &self.formal_font,
            Locales::zhTW => &self.taiwan_font,
            Locales::deDE => &self.formal_font,
            Locales::esES => &self.formal_font,
            Locales::frFR => &self.formal_font,
            Locales::itIT => &self.formal_font,
            Locales::koKR => &self.korean_font,
            Locales::plPL => &self.formal_font,
            Locales::enBG => &self.formal_font,
            Locales::Unknown => &self.formal_font,
        }
    }
}