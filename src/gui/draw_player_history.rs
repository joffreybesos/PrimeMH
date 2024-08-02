use std::collections::HashSet;

use notan::draw::*;
use notan::math::Rect;
use notan::prelude::*;

pub fn draw_player_history(player_pos: (f32, f32), player_pos_history: &HashSet<(u32, u32)>, draw: &mut Draw, scale: f32, width: &f32, height: &f32) {
    let size = (1.8, 0.5);
    for pos_history in player_pos_history {
        let npc_pos = transform_position(pos_history, size, player_pos, scale, width, height);
        let player_color = Color::from_hex(0x2087FDFF);
        draw_cross(npc_pos, size.0 * scale, player_color, 0.4 * scale, draw);
    }
}


fn transform_position(
    unit_pos: &(u32, u32),
    size: (f32, f32),
    player_pos: (f32, f32),
    scale: f32,
    width: &f32, 
    height: &f32
) -> (f32, f32) {
    let xdiff = unit_pos.0 as f32 - player_pos.0;
    let ydiff = unit_pos.1 as f32 - player_pos.1;

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
    draw.circle(150.0)
    .position(pos_x, pos_y)
    .color(Color::RED)
    .alpha(0.009)
    .scale_from((pos_x, pos_y), (1.0, 0.5));
}

