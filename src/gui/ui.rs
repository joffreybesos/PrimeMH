use notan::egui::{self, *};
use notan::math::{Mat3, Vec2};
use notan::prelude::*;
use notan::{draw::*, extra};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

use winapi::um::winuser::{
    SetWindowLongW, GWL_EXSTYLE, GWL_STYLE, WS_BORDER, WS_CAPTION, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_ACCEPTFILES,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_EX_WINDOWEDGE, WS_MINIMIZEBOX, WS_SYSMENU, WS_VISIBLE,
};

use crate::gui::draw_map::draw_map;
use crate::gui::Fonts;
use crate::localisation::localisation::{load_localisation_data, Localisation};
use crate::mapgeneration::blacha::is_blacha_ok;
use crate::memory::gamedata;
use crate::memory::process::D2RWindowArea;
use crate::settings::MapPosition;
use crate::{
    mapgeneration::{self, jsondata::SeedData},
    memory::{gamedata::GameData, process::D2RInstance},
    settings::Settings,
};
use winapi::shared::windef::{HWND, POINT};

use super::draw_lines::draw_lines;
use super::draw_objects::draw_objects;
use super::draw_player_history::draw_player_history;
use super::draw_presets::draw_presets;
use super::draw_units::draw_units;
use super::images;
use super::util::get_attached_levels;

#[notan_main]
pub fn start_ui() -> Result<(), String> {
    // load config
    let mut settings: Settings = match Settings::new() {
        Ok(settings) => settings,
        Err(err) => panic!("Error reading from settings file {}", err),
    };
    settings.detect_locale();

    let win_config = WindowConfig::default()
        .set_size(10, 10)
        .set_always_on_top(settings.general.overlay_mode)
        .set_decorations(!settings.general.overlay_mode)
        .set_mouse_passthrough(settings.general.overlay_mode)
        .set_transparent(settings.general.overlay_mode)
        .set_resizable(!settings.general.overlay_mode)
        .set_multisampling(settings.general.multisampling)
        .set_window_icon(Some("primemh.png".into()))
        .set_taskbar_icon(Some("primemh.png".into()))
        .set_title("PrimeMH")
        .set_high_dpi(settings.general.high_dpi)
        .set_vsync(settings.general.vsync);

    notan::init_with(init)
        .add_config(win_config)
        .add_config(DrawConfig)
        .add_config(EguiConfig)
        .add_plugin(extra::FpsLimit::new(settings.general.fps_limit))
        .update(update)
        .draw(draw)
        .build()
}

fn init(gfx: &mut Graphics) -> State {
    // load config
    let settings: Settings = match Settings::new() {
        Ok(settings) => settings,
        Err(err) => {
            log::error!("Error reading from settings file {}", err);
            panic!("Error reading from settings file {}", err)
        }
    };
    log::info!("Loaded settings file");

    let _blacha_result = is_blacha_ok(&settings).unwrap();
    log::info!("D2LoD test passed");

    let images = images::load_images(gfx);
    log::info!("Loaded images");

    let localisation: Localisation = load_localisation_data(&settings.general.language);

    let d2rprocess = D2RInstance::open_title(settings.general.title.clone());

    let exocet_font = gfx.create_font(include_bytes!("./fonts/exocet.otf")).unwrap();
    let formal_font = gfx.create_font(include_bytes!("./fonts/formal.otf")).unwrap();
    let korean_font = gfx
        .create_font(include_bytes!("./fonts/NotoSansCJKkr-Regular.otf"))
        .unwrap();
    let taiwan_font = gfx
        .create_font(include_bytes!("./fonts/NotoSansCJKtc-Regular.otf"))
        .unwrap();
    let blizzard_font = gfx
        .create_font(include_bytes!("./fonts/blizzardglobaltcunicode.ttf"))
        .unwrap();

    let fonts = Fonts {
        exocet_font,
        formal_font,
        korean_font,
        taiwan_font,
        blizzard_font,
    };

    log::info!("Loaded fonts");

    let seed_data = SeedData::default();

    State {
        d2rprocess,
        settings,
        seed_data,
        last_seed: 0,
        game_data: None,
        fonts,
        images,
        item_frame: 0,
        egui_rect: Rect {
            min: Pos2::ZERO,
            max: Pos2::ZERO,
        },
        egui_hovering: false,
        relative_mouse_pos: (0, 0),
        player_pos_history: HashSet::new(),
        launch_time: SystemTime::now(),
        localisation,
    }
}

#[derive(AppState)]
pub(crate) struct State {
    d2rprocess: D2RInstance,
    settings: Settings,
    seed_data: SeedData,
    last_seed: u32,
    game_data: Option<GameData>,
    fonts: Fonts,
    images: HashMap<String, Texture>,
    item_frame: i32,
    egui_rect: Rect,
    egui_hovering: bool,
    relative_mouse_pos: (i32, i32),
    player_pos_history: HashSet<(u32, u32)>,
    launch_time: SystemTime,
    localisation: Localisation,
}

fn update(app: &mut App, state: &mut State) {
    if let Some(game_data) = GameData::read_game_memory(&state.d2rprocess) {
        // if new seed
        if game_data.seed_values.map_seed != state.last_seed {
            // generate new seed data using blachas' tool and parse the JSON into seed_data
            log::info!(
                "New game detected, generating data for map seed {} {:?} {}",
                game_data.seed_values.map_seed,
                game_data.seed_values.difficulty,
                game_data.seed_values.level
            );
            log::info!("Using D2LoD path {}", &state.settings.general.d2lodpath.as_os_str().to_string_lossy());
            state.seed_data = mapgeneration::seeddata::generate_seed_data(&game_data.seed_values, &state.settings);
        }
        
        state.last_seed = game_data.seed_values.map_seed;
        let xval = game_data.player.pos_x as u32;
        let yval = game_data.player.pos_y as u32;
        state.game_data = Some(game_data);
        
        state.player_pos_history.insert((xval, yval));
    } else {
        state.player_pos_history.clear();
        state.game_data = None;
    }

    if state.settings.general.overlay_mode {
        // keep overlay aligned with game window
        let d2r_window = state.d2rprocess.get_window_info();
        app.window().set_size(d2r_window.width as u32, d2r_window.height as u32);
        app.window().set_position(d2r_window.x, d2r_window.y);
        let relative_mouse_pos = get_relative_mouse_pos(&d2r_window);
        if mouse_hovering_egui(relative_mouse_pos, state.egui_rect, app.window().dpi()) {
            if !state.egui_hovering {
                unsafe {
                    let hwnd = app.window().id() as isize as HWND;
                    let mut style =
                        WS_CAPTION | WS_MINIMIZEBOX | WS_BORDER | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_SYSMENU;
                    let mut style_ex = WS_EX_WINDOWEDGE | WS_EX_ACCEPTFILES;
                    style |= WS_VISIBLE;
                    style_ex |= WS_EX_TOPMOST;
                    SetWindowLongW(hwnd, GWL_STYLE, style as i32);
                    SetWindowLongW(hwnd, GWL_EXSTYLE, style_ex as i32);
                }
            }
            state.egui_hovering = true;
        } else {
            if state.egui_hovering {
                unsafe {
                    let hwnd = app.window().id() as isize as HWND;
                    let mut style =
                        WS_CAPTION | WS_MINIMIZEBOX | WS_BORDER | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_SYSMENU;
                    let mut style_ex = WS_EX_WINDOWEDGE | WS_EX_ACCEPTFILES;
                    style |= WS_VISIBLE;
                    style_ex |= WS_EX_TOPMOST;
                    style_ex |= WS_EX_TRANSPARENT | WS_EX_LAYERED;
                    SetWindowLongW(hwnd, GWL_STYLE, style as i32);
                    SetWindowLongW(hwnd, GWL_EXSTYLE, style_ex as i32);
                }
            }
            state.egui_hovering = false;
        }
        state.relative_mouse_pos = relative_mouse_pos;
    } else {
        if app.window().size().0 == 10 && app.window().size().1 == 10 {
            app.window().set_size(800, 600);
            app.window().set_position(100, 100);
        }
    }
}

fn draw(app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {
    if state.d2rprocess.is_window_active(app.window().id()) || !state.settings.general.overlay_mode {
        let mut output = plugins.egui(|ctx| {
            create_egui_panel(app, ctx, state);
            state.egui_rect = ctx.used_rect();
        });

        output.clear_color(Color::TRANSPARENT);

        let width: f32;
        let height: f32;
        let mut mask = gfx.create_draw();

        match state.settings.general.map_position {
            MapPosition::Center => {
                width = app.window().width() as f32;
                height = app.window().height() as f32;
                mask.rect((0.0, 0.0), (app.window().width() as f32, app.window().height() as f32));
            }
            MapPosition::TopLeft => {
                width = app.window().width() as f32 * 0.33;
                height = app.window().height() as f32 / 2.45;
                mask.rect((0.0, 0.0), (app.window().width() as f32 / 3.0, app.window().height() as f32 / 3.0));
            }
            MapPosition::TopRight => {
                width = app.window().width() as f32 * 1.67;
                height = app.window().height() as f32 / 2.44;
                mask.rect(
                    (app.window().width() as f32 - (app.window().width() as f32 / 3.0), 0.0),
                    (app.window().width() as f32 / 3.0, app.window().height() as f32 / 3.0),
                );
            }
        }

        let mut draw = gfx.create_draw();
        draw.mask(Some(&mask));
        let elapsed_time = SystemTime::now().duration_since(state.launch_time).expect("Fuck you!");
        if elapsed_time <= Duration::from_secs(5) {
            let splash_text = format!("Joffreybesos' Map overlay (PrimeMH)");
            draw.text(&state.fonts.blizzard_font, &splash_text)
                .position(app.window().width() as f32 * 0.5, app.window().height() as f32 * 0.1)
                .size(52.0)
                .color(Color::from_hex(0xC6B276FF))
                .h_align_center()
                .v_align_top();

            let splash_text = format!(
                "{}{}{}{}{}{}{}{}{}{}{}",
                obfstr::obfstr!("If you paid for this you have been scammed\n"),
                obfstr::obfstr!("如果您為此付出了，您已經被騙了\n"),
                obfstr::obfstr!("Wenn Sie dafür bezahlt haben, wurden Sie betrogen\n"),
                obfstr::obfstr!("Si pagaste por esto, has sido estafado\n"),
                obfstr::obfstr!("Si vous avez payé pour cela, vous avez été arnaque\n"),
                obfstr::obfstr!("Se hai pagato per questo sei stato truffato\n"),
                obfstr::obfstr!("당신이 이것을 지불했다면 당신은 사기를당했습니다\n"),
                obfstr::obfstr!("Jeśli za to zapłaciłeś, zostałeś oszukany\n"),
                obfstr::obfstr!("あなたがこれに対して支払った場合、あなたは詐欺されています\n"),
                obfstr::obfstr!("Se você pagou por isso, foi enganado\n"),
                obfstr::obfstr!("Если вы заплатили за это, вас обманули\n")
            );

            draw.text(&state.fonts.blizzard_font, &splash_text)
                .position(app.window().width() as f32 * 0.5, app.window().height() as f32 * 0.3)
                .size(22.0)
                .color(Color::from_hex(0xC6B276FF))
                .h_align_center()
                .v_align_top();
        } else {
            // in game
            if let Some(game_data) = &state.game_data {
                if (game_data.menus.automap_visible || state.settings.visual.always_show_map)
                    && !(state.settings.visual.hide_map_menus_open && game_data.menus.is_panel_open())
                {
                    let stitched_levels = get_attached_levels(&game_data.seed_values.level);
                    stitched_levels.iter().for_each(|level_id| {
                        if let Some(this_level) = state.seed_data.levels.iter_mut().find(|l| l.id == *level_id) {
                            let scale: f32 = state.settings.visual.scale;
                            // render map image here
                            if this_level.level_image.map_image.is_none() {
                                log::info!(
                                    "Rendering map image, seed: {}, difficulty: {:?}, level: {:?}",
                                    &game_data.seed_values.map_seed,
                                    &game_data.seed_values.difficulty,
                                    &this_level.name
                                );
                                this_level.level_image.map_image = Some(draw_map(gfx, this_level, &state.settings));
                            }
                            if let Some(map_image) = &mut this_level.level_image.map_image {
                                let render_scale = state.settings.general.render_scale;
                                let window_center_x = width as f32 * 0.5 / scale * render_scale;
                                let window_center_y = height as f32 * 0.5 / (scale / 2.0 / render_scale);

                                let map_position_x = ((this_level.offset.x as f32 - game_data.player.pos_x)
                                    * render_scale)
                                    + window_center_x;
                                let map_position_y = ((this_level.offset.y as f32 - game_data.player.pos_y)
                                    * render_scale)
                                    + window_center_y;

                                let player_pos_x = (game_data.player.pos_x - this_level.offset.x as f32) * render_scale;
                                let player_pos_y = (game_data.player.pos_y - this_level.offset.y as f32) * render_scale;
                                let scale_matrix =
                                    Mat3::from_scale(Vec2::from([scale / render_scale, scale / 2.0 / render_scale]));
                                draw.transform().push(scale_matrix);
                                draw.image(map_image)
                                    .translate(map_position_x, map_position_y)
                                    .rotate_degrees_from(
                                        (map_position_x + player_pos_x, map_position_y + player_pos_y),
                                        45.0,
                                    );

                                draw.transform().pop();
                                draw_player_history((game_data.player.pos_x, game_data.player.pos_y), &state.player_pos_history, &mut draw, scale, &width, &height);
                                draw_presets(&mut draw, this_level, &state.fonts.exocet_font, game_data, &state.settings, &state.images, &width, &height, &state.localisation);
                                draw_lines(&mut draw, this_level, game_data, &state.settings, &width, &height);
                            }
                        }
                    });

                    draw_units(
                        &mut draw,
                        game_data,
                        &state.settings,
                        &width,
                        &height,
                        &state.fonts,
                        &state.localisation,
                    );
                    draw_objects(&mut draw, game_data, &state.settings, &width, &height, &state.images);
                    draw.mask(None);

                    state.item_frame += 1;
                    if state.item_frame > 20 {
                        state.item_frame = 0;
                    }
                }
            } else {
                // in game menus

                let last_game_name = gamedata::get_last_game_name(&state.d2rprocess);

                if last_game_name.len() > 0 {
                    let last_game = format!("Last Game: {}", last_game_name);
                    draw.text(&state.fonts.exocet_font, &last_game)
                        .position(app.window().width() as f32 * 0.75, 10.0)
                        .size(16.0)
                        .color(Color::from_hex(0xC6B276FF))
                        .h_align_center()
                        .v_align_top();
                }
            }
        }

        let warning_duration = elapsed_time.as_secs() % 300;
        if warning_duration < 20 && elapsed_time >= Duration::from_secs(60) {
            let font_size = 12.0;
            let warning_text: String = format!("{}", obfstr::obfstr!("This is a free tool"));
            let warning_text2: String = format!("{}", obfstr::obfstr!("如果您為此付出了，您已經被騙"));
            let warning_text3: String =
                format!("{}", obfstr::obfstr!("당신이 이것을 지불했다면 당신은 사기를당했습니다"));

            draw.text(&state.fonts.blizzard_font, &warning_text)
                .position(app.window().width() as f32 - 5.0, 5.0)
                .size(font_size)
                .color(Color::from_hex(0xC6B276AA))
                .h_align_right()
                .v_align_top();

            draw.text(&state.fonts.blizzard_font, &warning_text2)
                .position(app.window().width() as f32 - 5.0, app.window().height() as f32)
                .size(font_size)
                .color(Color::from_hex(0xC6B276AA))
                .h_align_right()
                .v_align_bottom();

            draw.text(&state.fonts.blizzard_font, &warning_text3)
                .position(5.0, app.window().height() as f32)
                .size(font_size)
                .color(Color::from_hex(0xC6B276AA))
                .h_align_left()
                .v_align_bottom();
        }

        gfx.render(&output);
        gfx.render(&draw);
    } else {
        let mut draw = gfx.create_draw();
        draw.clear(Color::TRANSPARENT);
        gfx.render(&draw);
    }
}

fn create_egui_panel(app: &mut App, ctx: &Context, state: &mut State) {
    setup_custom_fonts(ctx);
    let translations = &state.localisation;
    ctx.set_pixels_per_point(app.window().dpi() as f32);
    egui::Window::new("D2R PrimeMH").default_open(false).show(ctx, |ui| {
        egui::CollapsingHeader::new(translations.get("map_settings"))
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(translations.get("map_scale"));
                        ui.add(egui::Slider::new(&mut state.settings.visual.scale, 0.1..=8.0));
                        ui.end_row();

                        ui.label(translations.get("map_opacity"));
                        ui.add(egui::Slider::new(&mut state.settings.visual.map_opacity, 0.0..=1.0));
                        ui.end_row();

                        ui.label(translations.get("always_show_map"));
                        ui.add(egui::Checkbox::new(&mut state.settings.visual.always_show_map, ""));
                        ui.end_row();

                        ui.label(translations.get("hide_map_in_menus"));
                        ui.add(egui::Checkbox::new(&mut state.settings.visual.hide_map_menus_open, ""));
                        ui.end_row();

                        ui.label(translations.get("map_position"));
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::Center,
                                translations.get("center"),
                            );
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::TopLeft,
                                translations.get("top_left"),
                            );
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::TopRight,
                                translations.get("top_right"),
                            );
                        });
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(translations.get("map_objects"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("objects_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(translations.get("show_chests"));
                        ui.add(egui::Checkbox::new(&mut state.settings.chests.enabled, ""));
                        ui.label(translations.get("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.chests.size)
                                .clamp_range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.end_row();

                        ui.label(translations.get("show_portals"));
                        ui.add(egui::Checkbox::new(&mut state.settings.portals.enabled, ""));
                        ui.label(translations.get("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.portals.size)
                                .clamp_range(0.1..=8.0)
                                .speed(0.1),
                        );
                        ui.end_row();

                        ui.label(translations.get("show_shrines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.shrines.enabled, ""));
                        ui.label(translations.get("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.shrines.size)
                                .clamp_range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.label(translations.get("text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.shrines.text_size)
                                .clamp_range(1.0..=20.0)
                                .speed(0.5),
                        );
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(translations.get("monsters"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("monsters_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(translations.get("show_immunities"));
                        ui.add(egui::Checkbox::new(&mut state.settings.monsters.immunities, ""));
                        ui.end_row();

                        ui.label(translations.get("normal_mobs"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.normal_mobs_size)
                                .clamp_range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.normal_mob_color);
                        ui.end_row();

                        ui.label(translations.get("minions"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.minions_mobs_size)
                                .clamp_range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.minions_mob_color);
                        ui.end_row();

                        ui.label(translations.get("champions"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.champions_mobs_size)
                                .clamp_range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.champions_mob_color);
                        ui.end_row();

                        ui.label(translations.get("uniques"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.unique_mobs_size)
                                .clamp_range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.unique_mob_color);
                        ui.end_row();

                        ui.label(translations.get("bosses"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.boss_mobs_size)
                                .clamp_range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.boss_mob_color);
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(translations.get("lines_and_pathfinding"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("lines_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(translations.get("show_waypoint_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.waypoint_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.waypoint_rgba);
                        ui.end_row();

                        ui.label(translations.get("show_exit_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.exit_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.exit_rgba);
                        ui.end_row();

                        ui.label(translations.get("show_quest_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.quest_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.quest_rgba);
                        ui.end_row();

                        ui.label(translations.get("show_boss_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.boss_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.boss_rgba);
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(translations.get("missiles"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("missiles_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(translations.get("show_missiles"));
                        ui.add(egui::Checkbox::new(&mut state.settings.missiles.enabled, ""));
                        ui.end_row();

                        // Fire
                        ui.label(translations.get("fire"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.fire_size)
                                .clamp_range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.fire_color);
                        ui.end_row();

                        // Cold
                        ui.label(translations.get("cold"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.cold_size)
                                .clamp_range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.cold_color);
                        ui.end_row();

                        // Poison
                        ui.label(translations.get("poison"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.poison_size)
                                .clamp_range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.poison_color);
                        ui.end_row();

                        // Lightning
                        ui.label(translations.get("lightning"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.lightning_size)
                                .clamp_range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.lightning_color);
                        ui.end_row();

                        // Physical
                        ui.label(translations.get("physical"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.physical_size)
                                .clamp_range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.physical_color);
                        ui.end_row();

                        // Magic
                        ui.label(translations.get("magic"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.magic_size)
                                .clamp_range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.magic_color);
                        ui.end_row();
                    });
            });

        ui.separator();
        ui.hyperlink("https://discord.gg/swswCBXbp6");

        let splash_text = format!(
            "{}{}{}{}{}",
            obfstr::obfstr!("If you paid for this, you have been scammed\n"),
            obfstr::obfstr!("如果您為此付出了，您已經被騙了\n"),
            obfstr::obfstr!("당신이 이것을 지불했다면 당신은 사기를당했습니다\n"),
            obfstr::obfstr!("あなたがこれに対して支払った場合、あなたは詐欺されています\n"),
            obfstr::obfstr!("Если вы заплатили за это, вас обманули\n")
        );
        ui.label(splash_text);

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(translations.get("quit")).clicked() {
                app.exit();
            }
            if ui.button(translations.get("save_settings")).clicked() {
                state.settings.save();
            }
        });
    });
}

fn get_relative_mouse_pos(d2r_window: &D2RWindowArea) -> (i32, i32) {
    let mut point = POINT { x: 0, y: 0 };
    unsafe { ::winapi::um::winuser::GetCursorPos(&mut point as *mut POINT) };
    (point.x - d2r_window.x, point.y - d2r_window.y)
}

fn mouse_hovering_egui(relative_mouse_pos: (i32, i32), egui_rect: Rect, dpi: f64) -> bool {
    relative_mouse_pos.0 > egui_rect.left() as i32
        && relative_mouse_pos.0 < (egui_rect.right() * dpi as f32) as i32
        && relative_mouse_pos.1 > egui_rect.top() as i32
        && relative_mouse_pos.1 < (egui_rect.bottom() * dpi as f32) as i32
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "blizzard".to_owned(),
        egui::FontData::from_static(include_bytes!("./fonts/blizzardglobaltcunicode.ttf")),
    );
    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "blizzard".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("blizzard".to_owned());
    ctx.set_fonts(fonts);
}
