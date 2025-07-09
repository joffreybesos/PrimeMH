use std::collections::HashSet;

use notan::draw::*;
use notan::math::Rect;
use notan::prelude::*;


use crate::localisation::localisation::detect_safe_font;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;
use crate::types::enchants::MonsterEnchants;
use crate::types::{
    missile::{MissileType, MissileUnit},
    npc::{MonsterFlag, NPC, NPCMode, NPCType, NPCUnit},
    player::PlayerUnit,
    roster::RosterItem,
    states::State,
    stats::Immunity,
};
use crate::LOCALISATION;

use super::util::get_attached_levels;
use super::Fonts;

pub fn draw_units(draw: &mut Draw, game_data: &GameData, settings: &Settings, width: &f32, height: &f32, fonts: &Fonts) {
    let player_pos = (game_data.player.pos_x, game_data.player.pos_y);

    // draw player dot at the centre
    draw_player(&game_data.player, player_pos, draw, settings.visual.scale, width, height);

    
    // draw npcs
    if settings.monsters.enabled {
        game_data.npcs.iter().for_each(|npc| match npc.npc_type {
            NPCType::Monster => { draw_monster(npc, player_pos, draw, settings, width, height, fonts); }
            NPCType::Town => { draw_town_npc(npc, player_pos, draw, settings, fonts, width, height); }
            NPCType::Pet => { draw_pet(npc, player_pos, draw, settings.visual.scale, width, height);}
            _ => (),
        });
    }

    // draw bosses separately to ensure they draw on top
    if settings.monsters.enabled {
        game_data.npcs.iter().for_each(|npc| {
            if let NPCType::Boss = npc.npc_type {
                draw_boss(npc, player_pos, draw, settings, fonts, width, height);
            }
        });
    }

    // draw other players
    // get players that are in the roster, but not the unit table
    let same_levels = get_attached_levels(&game_data.seed_values.level);
    let roster_players: Vec<&RosterItem> = game_data
        .roster_items
        .iter()
        .filter(|other_player| {
            !game_data
                .players
                .iter()
                .any(|player| player.unit_id == other_player.unit_id && same_levels.contains(&(other_player.area as u32)))
        })
        .collect();

    let hostile_unit_ids = is_hostile(&game_data.roster_items, game_data.player.unit_id);

    // if other players don't exist on unit tables, use roster data to draw them on map
    roster_players.iter().for_each(|other_player| {
        if game_data.player.unit_id != other_player.unit_id && same_levels.contains(&(other_player.area as u32)){
            let is_hostile = hostile_unit_ids.iter().any(|h| *h == other_player.unit_id);
            draw_other_player(
                (other_player.pos_x as f32, other_player.pos_y as f32),
                &other_player.name,
                player_pos,
                false,
                draw,
                settings.visual.scale,
                fonts,
                width, 
                height,
                is_hostile
            );
        }
    });

    // draw other players which are on the unit table
    game_data.players.iter().for_each(|other_player| {
        if game_data.player.unit_id != other_player.unit_id {
            if other_player.states[State::SharedStash as usize] != State::SharedStash {
                if other_player.pos_x > 0.0 && other_player.pos_y > 0.0 {
                    let is_hostile = hostile_unit_ids.iter().any(|h| h == &other_player.unit_id);
                    draw_other_player(
                        (other_player.pos_x, other_player.pos_y),
                        &other_player.player_name,
                        player_pos,
                        other_player.is_corpse,
                        draw,
                        settings.visual.scale,
                        fonts,
                        width, 
                        height,
                        is_hostile
                    );
                }
            }
        }
    });

    // draw missiles
    if settings.missiles.enabled {
        game_data.missiles.iter().for_each(|missile: &MissileUnit| {
            draw_missle_type(missile, player_pos, draw, settings, width, height);
        });
    }
}

fn is_hostile(roster_players: &Vec<RosterItem>, player_unit_id: u32) -> Vec<u32> {
    let player = roster_players.iter().find(|op| op.unit_id == player_unit_id);
    let mut hostile_unit_ids: Vec<u32> = vec![];
    match player {
        Some(p) => {
            p.hostile_info.iter().for_each(|h| { 
                if h.hostile_flag > 0 {
                    hostile_unit_ids.push(h.dw_unit_id);
                }
            })
        },
        None => (),
    }
    return hostile_unit_ids;
}

fn draw_monster(npc: &NPCUnit, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32, all_fonts: &Fonts) {
    if !(npc.mode != NPCMode::Dead && npc.mode != NPCMode::Death) | npc.states.contains(&State::Revive) {
        return;
    }
    let scale = settings.visual.scale;
    let unique_size: f32 = settings.monsters.unique_mobs_size * scale;
    let champion_size: f32 = settings.monsters.champions_mobs_size * scale;
    let minion_mob_size: f32 = settings.monsters.minions_mobs_size * scale;
    let normal_mob_size: f32 = settings.monsters.normal_mobs_size * scale;

    // should precalculate this somewhere
    let unique_color: Color = convert_color(settings.monsters.unique_mob_color);
    let champion_color: Color = convert_color(settings.monsters.champions_mob_color);
    let minion_color: Color = convert_color(settings.monsters.minions_mob_color);
    let normal_color: Color = convert_color(settings.monsters.normal_mob_color);

    let immunities = npc.get_immunities();
    let unit_pos = (npc.pos_x, npc.pos_y);
    let (size, npc_pos, mob_color) = match npc.monster_flag {
        MonsterFlag::SuperUnique | MonsterFlag::Unique => {
            let size = (unique_size, unique_size / 1.0);
            let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
            (size, npc_pos, unique_color)
        }
        MonsterFlag::Champion => {
            let size = (champion_size, champion_size / 1.0);
            let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
            (size, npc_pos, champion_color)
        }
        MonsterFlag::Minion => {
            let size = (minion_mob_size, minion_mob_size / 1.0);
            let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
            (size, npc_pos, minion_color)
        }
        _ => {
            let size = (normal_mob_size, normal_mob_size / 1.0);
            let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
            (size, npc_pos, normal_color)
        }
    };
    if immunities.is_empty() || !settings.monsters.immunities {
        draw.ellipse(npc_pos, size).color(mob_color).scale_from(npc_pos, (1.0, 0.5));
    } else {
        draw_monster_with_immunities(draw, immunities, npc_pos, size, mob_color);
    }
    draw_monster_enchants(draw, &npc.monster_enchants, npc_pos, size.0 * 1.5, all_fonts);
}

fn draw_monster_enchants(
    draw: &mut Draw,
    enchants: &Vec<MonsterEnchants>,
    npc_pos: (f32, f32),
    size: f32,
    all_fonts: &Fonts
) {
    
    if enchants.contains(&MonsterEnchants::Cursed) {
        // let x = npc_pos.0;
        // let y = npc_pos.1;
        // let scale_x = size / 12.0;
        // let scale_y = scale_x / 2.0;
    
        // draw.path()
        //     .move_to(x, y)
        //     .move_to((13.666 * scale_x) + x, (-6.0 * scale_y)+y)
        //     .line_to((-9.5 * scale_x) + x, (10.0 * scale_y)+y)
        //     .line_to((0.0 * scale_x) + x, (-16.0 * scale_y)+y)
        //     .line_to((9.7 * scale_x) + x, (10.0 * scale_y)+y)
        //     .line_to((-13.666 * scale_x) + x, (-6.0 * scale_y)+y)
        //     .line_to((13.666 * scale_x) + x, (-6.0 * scale_y)+y)
            
    
        //     .color(Color::BLACK)
        //     .stroke(2.0);
        let text_pos = (npc_pos.0, (npc_pos.1 - (size*1.5)));
        draw.text(&all_fonts.blizzard_font, "C")
            .position(text_pos.0 + 1.0, text_pos.1 + 1.0)
            .size(size)
            .color(Color::BLACK)
            .h_align_center()
            .v_align_top();
        draw.text(&all_fonts.blizzard_font, "C")
            .position(text_pos.0, text_pos.1)
            .size(size)
            .color(Color::from_rgb(212.0,175.0,55.0))
            .h_align_center()
            .v_align_top();
    }
    
}

fn draw_monster_with_immunities(
    draw: &mut Draw,
    immunities: HashSet<Immunity>,
    npc_pos: (f32, f32),
    size: (f32, f32),
    mob_color: Color
) {
    draw_immunities(npc_pos, size, immunities, draw, mob_color);

    draw.ellipse(npc_pos, size)
        .stroke(size.0 / 3.0)
        .color(mob_color.with_alpha(1.0))
        .alpha(mob_color.a)
        .fill()
        .scale_from(npc_pos, (1.0, 0.5));
    
}


fn draw_player(player: &PlayerUnit, player_pos: (f32, f32), draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    let size = (1.8, 0.5);
    let unit_pos = (player.pos_x, player.pos_y);
    let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    let player_color = Color::from_hex(0x2087FDFF);
    draw_cross(npc_pos, size.0 * scale, player_color, 0.4 * scale, draw);
}

fn draw_other_player(
    unit_pos: (f32, f32),
    player_name: &String,
    player_pos: (f32, f32),
    is_corpse: bool,
    draw: &mut Draw,
    scale: f32,
    all_fonts: &Fonts,
    width: &f32, 
    height: &f32,
    is_hostile: bool,
) {
    let size = (1.8, 0.5);
    let other_player_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    let color: Color = if is_corpse { Color::MAGENTA } else { if is_hostile { Color::RED } else { Color::GREEN } };

    draw_cross(other_player_pos, size.0 * scale, color, 0.4 * scale, draw);

    match detect_safe_font(player_name.clone(), all_fonts) {
        Some(font) => {
            
            // there is a bug drawing non-english chars, it doesn't seem to be encoding though
            let text_pos = (other_player_pos.0, (other_player_pos.1 - (7.0 * scale)));
            draw.text(&font, player_name)
                .position(text_pos.0 + 1.5, text_pos.1 + 1.5)
                .size(5.0 * scale)
                .color(Color::BLACK)
                .h_align_center()
                .v_align_top();
            draw.text(&font, player_name)
                .position(text_pos.0, text_pos.1)
                .size(5.0 * scale)
                .color(color)
                .h_align_center()
                .v_align_top();
        },
        None => ()
    };
}

fn draw_town_npc(npc: &NPCUnit, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, all_fonts: &Fonts,  width: &f32, height: &f32) {
    let scale = settings.visual.scale;
    let size = (1.8, 0.5);
    let unit_pos = (npc.pos_x, npc.pos_y);
    let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    let color = Color::WHITE;
    draw_cross(npc_pos, size.0 * scale, color, 0.4 * scale, draw);
    
    let localisation = LOCALISATION.lock().unwrap();
    let npc_name = format!("{:?}", npc.txt_file_no);
    let npc_label: String = localisation.get_npc_name(&npc_name);
    draw_npc_name(npc_pos, size.1, &npc_label, draw, settings, scale, all_fonts);
}

fn draw_boss(npc: &NPCUnit, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, all_fonts: &Fonts, width: &f32, height: &f32) {
    if npc.mode != NPCMode::Dead && npc.mode != NPCMode::Death {
        let scale = settings.visual.scale;
        let boss_color: Color = convert_color(settings.monsters.boss_mob_color);
        let size = (settings.monsters.boss_mobs_size * scale, settings.monsters.boss_mobs_size * scale / 2.0);
        let unit_pos = (npc.pos_x, npc.pos_y);
        let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);

        let immunities = npc.get_immunities();
        if !immunities.is_empty() || settings.monsters.immunities {
            draw_immunities(npc_pos, size, immunities, draw, boss_color);
        }

        draw.ellipse(npc_pos, size).color(boss_color);

        match npc.get_health() {
            Some((health, max_health)) => {
                let localisation = LOCALISATION.lock().unwrap();
                let font = all_fonts.get_safe_font(&settings.general.language);
                let hp_percent = health as f32 / max_health as f32;
                let boss_text: String = format!("{:?}", npc.txt_file_no);
                let npc_label: String = localisation.get_npc_name(&boss_text);
                draw_health_bar(npc_pos, size.1, hp_percent, npc_label, draw, settings, 0.0, font);
            },
            None => (),
        }
    }
}

fn draw_npc_name(
    npc_pos: (f32, f32),
    size: f32,
    text: &String,
    draw: &mut Draw,
    settings: &Settings,
    scale: f32,
    all_fonts: &Fonts, 
) {
    let font_size = 4.5;
    let npc_name_pos = (npc_pos.0, npc_pos.1 - ((size + 1.0) * scale * 3.2));
    let font = all_fonts.get_safe_font_formal(&settings.general.language);

    draw.text(&font, &text)
        .position(npc_name_pos.0, npc_name_pos.1)
        .size(font_size * scale)
        .color(Color::TRANSPARENT)
        .h_align_center()
        .v_align_middle();

    let bounds: Rect = draw.last_text_bounds();
    let pad = 1.0;

    draw.rect(
        (bounds.x - (pad * scale), bounds.y - pad),
        (bounds.width + (pad * scale * 2.0), bounds.height + pad + pad),
    )
    .color(Color::from_hex(0x00000088));

    draw.text(&font, &text)
        .position(npc_name_pos.0 + 1.0, npc_name_pos.1 + 1.0)
        .size(font_size * scale)
        .color(Color::BLACK)
        .h_align_center()
        .v_align_middle();
    draw.text(&font, &text)
        .position(npc_name_pos.0, npc_name_pos.1)
        .size(font_size * scale)
        .color(Color::from_hex(0xc6b276FF))
        .h_align_center()
        .v_align_middle();
}

pub fn draw_health_bar(
    npc_pos: (f32, f32),
    size: f32,
    hp_percent: f32,
    text: String,
    draw: &mut Draw,
    settings: &Settings,
    vertical_offset: f32,
    font: &Font
) {
    let font_size = 4.5;
    // draw boss health bar
    let scale = settings.visual.scale;
    let health_bar_pos = (npc_pos.0, npc_pos.1 - (size * scale) - vertical_offset);

    draw.text(font, &text)
        .position(health_bar_pos.0, health_bar_pos.1)
        .size(font_size * scale)
        .color(Color::TRANSPARENT)
        .h_align_center()
        .v_align_middle();

    let bounds: Rect = draw.last_text_bounds();
    let pad = 1.0;

    draw.rect(
        (bounds.x - (pad * scale), bounds.y - pad),
        (bounds.width + (pad * scale * 2.0), bounds.height + pad + pad),
    )
    .color(Color::from_hex(0x00000088));
    draw.rect(
        (bounds.x - (pad * scale), bounds.y - pad),
        ((bounds.width + (pad * scale * 2.0)) * hp_percent, bounds.height + pad + pad),
    )
    .color(Color::from_hex(0xaa000088));

    draw.text(font, &text)
        .position(health_bar_pos.0 + 1.0, health_bar_pos.1 + 1.0)
        .size(font_size * scale)
        .color(Color::BLACK)
        .h_align_center()
        .v_align_middle();
    draw.text(font, &text)
        .position(health_bar_pos.0, health_bar_pos.1)
        .size(font_size * scale)
        .color(Color::from_hex(0xD4AF37FF))
        .h_align_center()
        .v_align_middle();
}

fn draw_pet(npc: &NPCUnit, player_pos: (f32, f32), draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    if !(npc.mode != NPCMode::Dead && npc.mode != NPCMode::Death) {
        return;
    }
    let size = (3.0, 1.75);
    let unit_pos = (npc.pos_x, npc.pos_y);
    let npc_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    match &npc.txt_file_no {
        NPC::Hydra | NPC::Hydra2 | NPC::Hydra3 => {
            draw.ellipse(npc_pos, size).color(Color::RED);
        }
        _ => {
            draw_cross(npc_pos, 1.8 * scale, Color::from_hex(0x436f73ff), 0.2 * scale, draw);
        }
    };
}

fn draw_missle_type(missile: &MissileUnit, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, width: &f32, height: &f32) {
    let scale = settings.visual.scale;
    let (size, color) = match missile.missile_type {
        MissileType::Fire => (settings.missiles.fire_size, convert_color(settings.missiles.fire_color)),
        MissileType::Ice => (settings.missiles.cold_size, convert_color(settings.missiles.cold_color)),
        MissileType::Lightning => (settings.missiles.lightning_size, convert_color(settings.missiles.lightning_color)),
        MissileType::Poison => (settings.missiles.poison_size, convert_color(settings.missiles.poison_color)),
        MissileType::Magic => (settings.missiles.magic_size, convert_color(settings.missiles.magic_color)),
        MissileType::Physical => (settings.missiles.physical_size, convert_color(settings.missiles.physical_color)),
        _ => (0.0, Color::TRANSPARENT),
    };
    let unit_pos = (missile.pos_x, missile.pos_y);
    let missile_pos = transform_position(unit_pos, (size, size / 2.0), player_pos, scale, width, height);
    draw.ellipse(missile_pos, (size, size / 2.0)).color(color);
    //missile overlay tint
    // draw.ellipse(missile_pos, size).color(Color::from_hex(missile.missile_color));
}

fn transform_position(
    unit_pos: (f32, f32),
    size: (f32, f32),
    player_pos: (f32, f32),
    scale: f32,
    width: &f32, 
    height: &f32
) -> (f32, f32) {
    let xdiff = unit_pos.0 - player_pos.0;
    let ydiff = unit_pos.1 - player_pos.1;

    let center_x = *width as f32 / 2.0;
    let center_y = *height as f32 / 2.0;
    let angle: f32 = std::f32::consts::FRAC_PI_4;
    let x = xdiff * angle.cos() - ydiff * angle.sin();
    let y = xdiff * angle.sin() + ydiff * angle.cos();

    let new_pos_x = center_x + (x * scale) - (size.0 / 2.0);
    let new_pos_y = center_y + (y * scale * 0.5) - (size.1 / 2.0);

    (new_pos_x, new_pos_y)
}

fn draw_cross(pos: (f32, f32), cross_size: f32, color: Color, stroke: f32, draw: &mut Draw) {
    let pos_x = pos.0;
    let pos_y = pos.1;
    draw.path()
        .move_to(pos_x, pos_y + cross_size)
        .line_to(pos_x + cross_size, pos_y + cross_size + cross_size)
        .line_to(pos_x + cross_size + cross_size, pos_y + cross_size)
        .line_to(pos_x + cross_size, pos_y)
        .line_to(pos_x + cross_size + cross_size, pos_y - cross_size)
        .line_to(pos_x + cross_size, pos_y - cross_size - cross_size)
        .line_to(pos_x, pos_y - cross_size)
        .line_to(pos_x - cross_size, pos_y - cross_size - cross_size)
        .line_to(pos_x - cross_size - cross_size, pos_y - cross_size)
        .line_to(pos_x - cross_size, pos_y)
        .line_to(pos_x - cross_size - cross_size, pos_y + cross_size)
        .line_to(pos_x - cross_size, pos_y + cross_size + cross_size)
        .line_to(pos_x, pos_y + cross_size)
        .color(color)
        .stroke(stroke)
        .scale_from((pos_x, pos_y), (1.0, 0.5));
}

fn convert_color(color_arr: [u8; 4]) -> Color {
    Color::from_bytes(color_arr[0], color_arr[1], color_arr[2], color_arr[3])
}


fn draw_immunities(npc_pos: (f32, f32), size: (f32, f32), immunities: HashSet<Immunity>, draw: &mut Draw, mob_color: Color) {

    let mut colors = vec![];
    immunities.iter().for_each(|immunity| match immunity {
        Immunity::Physical => colors.push(Color::from_hex(0xCD853FFF).with_alpha(mob_color.a)),
        Immunity::Magic => colors.push(Color::from_hex(0xFF8800FF).with_alpha(mob_color.a)),
        Immunity::Fire => colors.push(Color::from_hex(0xFF0000FF).with_alpha(mob_color.a)),
        Immunity::Lightning => colors.push(Color::from_hex(0xE0E000FF).with_alpha(mob_color.a)),
        Immunity::Cold => colors.push(Color::from_hex(0x0000FFFF).with_alpha(mob_color.a)),
        Immunity::Poison => colors.push(Color::from_hex(0x32CD32FF).with_alpha(mob_color.a)),
        Immunity::None => colors.push(Color::from_hex(0x00000000).with_alpha(mob_color.a)),
    });
    if colors.len() == 0 {
        return; // no immunities to draw
    }
    colors.sort_by_key(|c| c.hex());
    
    

    let degrees = 360 / colors.len();
    for (im, color) in colors.iter().enumerate() {
        let x: f32 = npc_pos.0;  // x-coordinate of center of arc
        let y: f32 = npc_pos.1;  // y-coordinate of center of arc
        let radius: f32 = size.0 * 1.6;  // radius of arc
        let start_angle: f32 = 270.0 + (im * degrees) as f32;
        let end_angle: f32 = 270.0 + ((im + 1) * degrees) as f32;
        {
            let start_x = x + radius * (start_angle as f32 * std::f32::consts::PI / 180.0).cos();
            let start_y = y + radius * (start_angle as f32 * std::f32::consts::PI / 180.0).sin();
            let mut binding = draw.path();
            let path = binding.move_to(x,y);
            path.line_to(start_x, start_y);
            for degrees in ((start_angle as u32)..(end_angle as u32)).step_by(8) {
                let x1 = x + radius * (degrees as f32 * std::f32::consts::PI / 180.0).cos();
                let y1 = y + radius * (degrees as f32 * std::f32::consts::PI / 180.0).sin();
                path.line_to(x1, y1);
                // draw.rect((x1, y1), (2.0, 2.0)).color(Color::GREEN);
            }
            let end_x = x + radius * (end_angle as f32 * std::f32::consts::PI / 180.0).cos();
            let end_y = y + radius * (end_angle as f32 * std::f32::consts::PI / 180.0).sin();
            path.line_to(end_x, end_y);
            path.line_to(x, y);
            path.fill();
            path.color(*color).stroke(1.0);
            path.scale_from(npc_pos, (1.0, 0.5));
        }
    }
    
}
