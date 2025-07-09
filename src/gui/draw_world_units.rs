use notan::draw::*;
use notan::prelude::*;

use crate::gui::draw_units::draw_health_bar;
use crate::gui::Fonts;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;
use crate::types::npc::NPCUnit;



pub fn draw_world_health_bars(draw: &mut Draw, game_data: &GameData, settings: &Settings, width: &f32, height: &f32, all_fonts: &Fonts) {
    let player_pos = (game_data.player.pos_x, game_data.player.pos_y);
    game_data.npcs.iter().for_each(|npc| {
        draw_world_health_bar(draw, npc, player_pos, settings, width, height, all_fonts);
    });
    
}

fn draw_world_health_bar(draw: &mut Draw, npc: &NPCUnit, player_pos: (f32, f32), settings: &Settings, width: &f32, height: &f32, all_fonts: &Fonts) {
    let size = (2.0, 2.0 / 1.0);
    let scale = 27.0;
    let npc_pos = transform_position((npc.pos_x, npc.pos_y), player_pos, scale, width, height);
    match npc.get_health() {
        Some((health, max_health)) => {
            // let localisation = LOCALISATION.lock().unwrap();
            let font = all_fonts.get_safe_font(&settings.general.language);
            let hp_percent = health as f32 / max_health as f32;
            let boss_text: String = format!("{:?}", npc.txt_file_no);
            // let npc_label: String = localisation.get_npc_name(&boss_text);
            
            draw_health_bar(npc_pos, size.1, hp_percent, boss_text, draw, settings, 90.0, font);
        },
        None => (),
    }

}

fn convert_color(color_arr: [u8; 4]) -> Color {
    Color::from_bytes(color_arr[0], color_arr[1], color_arr[2], color_arr[3])
}

fn transform_position(unit_pos: (f32, f32), player_pos: (f32, f32), scale: f32, width: &f32, height: &f32) -> (f32, f32) {
    let xdiff = unit_pos.0 - player_pos.0;
    let ydiff = unit_pos.1 - player_pos.1;

    let center_x = *width as f32 / 2.0;
    let center_y = *height as f32 / 2.0;
    let angle: f32 = std::f32::consts::FRAC_PI_4;
    let x = xdiff * angle.cos() - ydiff * angle.sin();
    let y = xdiff * angle.sin() + ydiff * angle.cos();

    let new_pos_x = center_x + (x * scale);
    let new_pos_y = center_y + (y * scale * 0.5);

    (new_pos_x, new_pos_y)
}
