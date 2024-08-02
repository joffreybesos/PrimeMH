use std::collections::HashMap;

use notan::draw::*;
use notan::prelude::*;

use crate::localisation::localisation::Localisation;
use crate::mapgeneration::jsondata::LevelData;
use crate::mapgeneration::pois::POIType;
use crate::mapgeneration::pois::POI;
use crate::memory::gamedata::GameData;
use crate::settings::Settings;
use crate::types::object::GameObjectMode;
use crate::types::object::GameObjectType;

// this will draw the preset POI data from the generated map data
// this includes waypoints, exits, certain shrines, super chests, NPC spawn locations
pub fn draw_presets(
    draw: &mut Draw,
    this_level: &mut LevelData,
    exocet_font: &Font,
    game_data: &GameData,
    settings: &Settings,
    images: &HashMap<String, Texture>,
    width: &f32,
    height: &f32,
    localisation: &Localisation
) {
    let player_pos = (game_data.player.pos_x, game_data.player.pos_y);
    let current_level_id = game_data.seed_values.level;

    let shrine_image = images.get("shrine").unwrap();
    let well_image = images.get("well").unwrap();
    let super_chest_image = images.get("superchest").unwrap();

    for poi in &mut this_level.level_image.pois.iter_mut() {
        for chest in game_data.objects.iter() {
            if chest.pos_x == poi.world_x && chest.pos_y == poi.world_y {
                // remove chests from presets that have been opened
                if (chest.object_type == GameObjectType::Chest || chest.object_type == GameObjectType::SuperChest)
                    && (chest.chest_state.is_none() || chest.mode != GameObjectMode::Neutral)
                {
                    poi.poi_type = POIType::Unknown;
                };
            }
        }
    }
    for shrine in game_data.objects.iter() {
        let mut found = false;
        for poi in &mut this_level.level_image.pois.iter_mut() {
            if shrine.object_type == GameObjectType::Shrine
                && shrine.pos_x == poi.world_x
                && shrine.pos_y == poi.world_y
            {
                found = true;
                poi.label = match shrine.shrine_type {
                    Some(a) => a.to_string(),
                    None => String::new(),
                };
            }
        }
        if !found {
            if shrine.shrine_type.is_some() {
                let label =  shrine.shrine_type.unwrap().to_string();
                let new_shrine = POI::new_shrine(shrine.pos_x, shrine.pos_y, &this_level.offset, label);
                this_level.level_image.pois.push(new_shrine);    
            }
        }
    }

    let pois = &this_level.level_image.pois;
    for poi in pois.iter() {
        match poi.poi_type {
            POIType::Waypoint => {
                draw_waypoint(poi, player_pos, draw, settings.visual.scale, width, height);
            }
            POIType::Shrine => {
                draw_shrine(poi, player_pos, draw, settings, exocet_font, shrine_image, width, height);
            }
            POIType::Well => {
                draw_shrine(poi, player_pos, draw, settings, exocet_font, well_image, width, height);
            }
            POIType::Chest => (),
            POIType::SuperChest => {
                draw_super_chest(poi, player_pos, draw, settings.visual.scale, super_chest_image, width, height);
            }
            POIType::Exit => {
                draw_exit(poi, player_pos, this_level, draw, exocet_font, settings.visual.scale, current_level_id, width, height, localisation);
            }
            POIType::GoodExit => {
                draw_good_exit(poi, player_pos, this_level, draw, settings.visual.scale, width, height);
            }
            POIType::QuestItem => {
                draw_quest_item(poi, player_pos, draw, settings.visual.scale, width, height);
            }
            POIType::NPCSpawn => {
                draw_npc_spawn(poi, player_pos, draw, settings.visual.scale, width, height);
            }
            POIType::Unknown => (),
        }
    }
}

fn draw_waypoint(poi: &POI, player_pos: (f32, f32), draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    let size = (6.0 * scale, 3.0 * scale);
    let unit_pos = (poi.world_x as f32, poi.world_y as f32);
    let poi_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    draw_diamond(draw, poi_pos, size, Color::YELLOW);
}

fn draw_exit(
    poi: &POI,
    player_pos: (f32, f32),
    this_level: &LevelData,
    draw: &mut Draw,
    font: &Font,
    scale: f32,
    current_level_id: u32,
    width: &f32, 
    height: &f32,
    localisation: &Localisation
) {
    if poi.id > 0 {
        let size = (6.0 * scale, 3.0 * scale);
        let unit_pos = (poi.world_x as f32, poi.world_y as f32);
        let poi_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
        let color = Color::from_rgb(255.0, 0.0, 255.0);
        draw_diamond(draw, poi_pos, size, color);

        let label: String = localisation.get_level(&poi.label);

        if current_level_id == this_level.id || poi.class != "walkable" {
            let text_pos = (poi_pos.0 + (size.0 / 2.0), (poi_pos.1 + (size.1 / 2.0)) - (10.0 * scale));
            //TODO: Fix the text here, need to figure out how to flip or mirror it
            draw.text(font, &label)
                .position(text_pos.0 + 2.0, text_pos.1 + 2.0)
                .size(6.0 * scale)
                .color(Color::BLACK)
                .h_align_center()
                .v_align_top();
            draw.text(font, &label)
                .position(text_pos.0, text_pos.1)
                .size(6.0 * scale)
                .color(Color::WHITE)
                .h_align_center()
                .v_align_top();
        }
    }
}

fn draw_good_exit(poi: &POI, player_pos: (f32, f32), _this_level: &LevelData, draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    let size = (8.0 * scale, 4.0 * scale);
    let unit_pos = (poi.world_x as f32, poi.world_y as f32);
    let poi_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    let color = Color::from_rgb(0.0, 255.0, 0.0);
    draw_diamond(draw, poi_pos, size, color);
}

fn draw_quest_item(poi: &POI, player_pos: (f32, f32), draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    let size = (1.0 * scale, 1.0 * scale);
    let unit_pos = (poi.world_x as f32, poi.world_y as f32);
    let poi_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    draw.rect(poi_pos, size).color(Color::from_rgb(0.0, 255.0, 0.0));
}

fn draw_super_chest(poi: &POI, player_pos: (f32, f32), draw: &mut Draw, scale: f32, super_chest_image: &Texture, width: &f32, height: &f32) {
    let size = (3.0 * scale, 3.0 * scale);
    let unit_pos = (poi.world_x as f32, poi.world_y as f32);
    let poi_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    let h = (super_chest_image.height() / 5.0) * scale;
    let w = (super_chest_image.width() / 5.0) * scale;
    draw.image(super_chest_image)
        .size(w, h)
        .position(poi_pos.0 - (w / 2.5), poi_pos.1 - (h / 1.5));
}

fn draw_npc_spawn(poi: &POI, player_pos: (f32, f32), draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    let size = (1.0 * scale, 1.0 * scale);
    let unit_pos = (poi.world_x as f32, poi.world_y as f32);
    let poi_pos = transform_position(unit_pos, size, player_pos, scale, width, height);
    draw.rect(poi_pos, size).color(Color::from_rgb(255.0, 0.0, 0.0));
}

fn draw_shrine(poi: &POI, player_pos: (f32, f32), draw: &mut Draw, settings: &Settings, font: &Font, image: &Texture, width: &f32, height: &f32) {
    if !settings.shrines.enabled {
        return;
    }
    let scale = settings.visual.scale;
    let unit_pos = (poi.world_x as f32, poi.world_y as f32);
    let h = (image.height() * settings.shrines.size) * scale;
    let w = (image.width() * settings.shrines.size) * scale;
    let poi_pos = transform_position(unit_pos, (h, w), player_pos, scale, width, height);
    draw.image(image)
        .size(w, h)
        .position(poi_pos.0 + (w / 1.5), poi_pos.1 - (h / 2.0));

    if poi.poi_type != POIType::Well {
        let text_pos = (poi_pos.0 + (w / 1.0), (poi_pos.1 - (10.0 * scale)));
        draw.text(font, &poi.label)
            .position(text_pos.0 + 1.5, text_pos.1 + 1.5)
            .size(settings.shrines.text_size * scale)
            .color(Color::BLACK)
            .h_align_center()
            .v_align_top();
        draw.text(font, &poi.label)
            .position(text_pos.0, text_pos.1)
            .size(settings.shrines.text_size * scale)
            .color(Color::from_hex(0xFFD700FF))
            .h_align_center()
            .v_align_top();
    }
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

fn draw_diamond(draw: &mut Draw, poi_pos: (f32, f32), size: (f32, f32), color: Color) {
    let pos_x = poi_pos.0 + (size.0 / 2.0);
    let pos_y = poi_pos.1 + (size.1 / 2.0);
    draw.path()
        .move_to(pos_x, pos_y + size.1)
        .line_to(pos_x + size.0, pos_y)
        .line_to(pos_x, pos_y - size.1)
        .line_to(pos_x - size.0, pos_y)
        .line_to(pos_x, pos_y + size.1)
        .color(color)
        .stroke(1.0)
        .fill();
}
