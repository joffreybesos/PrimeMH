use notan::text::Font;

mod draw_lines;
mod draw_map;
mod draw_objects;
mod draw_player_history;
mod draw_presets;
mod draw_units;
mod images;
pub mod ui;
mod util;

pub struct Fonts {
    exocet_font: Font,
    formal_font: Font,
    korean_font: Font,
    taiwan_font: Font,
    blizzard_font: Font,
}