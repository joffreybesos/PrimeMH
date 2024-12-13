use notan::egui::{self, *};
use notan::prelude::*;
use crate::settings::{Locales, MapPosition};
use crate::LOCALISATION;
use super::ui::State;


pub fn create_egui_panel(app: &mut App, ctx: &Context, state: &mut State) {
    setup_custom_fonts(ctx);
    let mut localisation = LOCALISATION.lock().unwrap();
    
    if localisation.current_locale != state.settings.general.language {
        localisation.update_locale(state.settings.general.language.clone());
    }
    ctx.set_pixels_per_point(app.window().dpi() as f32);
    egui::Window::new(egui::RichText::new("PrimeMH").size(16.0)).default_open(false).show(ctx, |ui| {
        egui::Grid::new("helper_text")
        .num_columns(2)
        .spacing([6.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                let toggle_text = format!(
                    "{}\n{}",
                    localisation.get_primemh("hide_ui"),
                    localisation.get_primemh("hide_map"),
                );
                ui.label(toggle_text);
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                
                if ui
                    .add(egui::Button::image(egui::Image::new(state.language_icon)
                        .max_width(25.0)
                        .tint(egui::Color32::GRAY)))
                    .clicked()
                {
                    state.language_selector_visible = true;
                }
            });
        });
        
        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("map_settings"))
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("map_scale"));
                        ui.add(egui::Slider::new(&mut state.settings.visual.scale, 0.1..=8.0));
                        ui.end_row();

                        ui.label(localisation.get_primemh("map_opacity"));
                        ui.add(egui::Slider::new(&mut state.settings.visual.map_opacity, 0.0..=1.0));
                        
                        ui.end_row();

                        ui.label(localisation.get_primemh("always_show_map"));
                        ui.add(egui::Checkbox::new(&mut state.settings.visual.always_show_map, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("hide_map_in_menus"));
                        ui.add(egui::Checkbox::new(&mut state.settings.visual.hide_map_menus_open, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("map_position"));
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::Center,
                                localisation.get_primemh("center"),
                            );
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::TopLeft,
                                localisation.get_primemh("top_left"),
                            );
                            ui.radio_value(
                                &mut state.settings.general.map_position,
                                MapPosition::TopRight,
                                localisation.get_primemh("top_right"),
                            );
                        });
                        ui.end_row();
                        
                        
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("map_objects"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("objects_grid")
                    .num_columns(2)
                    .spacing([6.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_chests"));
                        ui.add(egui::Checkbox::new(&mut state.settings.chests.enabled, ""));
                        ui.label(localisation.get_primemh("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.chests.size)
                                .range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_portals"));
                        ui.add(egui::Checkbox::new(&mut state.settings.portals.enabled, ""));
                        ui.label(localisation.get_primemh("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.portals.size)
                                .range(0.1..=8.0)
                                .speed(0.1),
                        );
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_shrines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.shrines.enabled, ""));
                        ui.label(localisation.get_primemh("size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.shrines.size)
                                .range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.label(localisation.get_primemh("text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.shrines.text_size)
                                .range(1.0..=20.0)
                                .speed(0.5),
                        );
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("monsters"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("monsters_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_immunities"));
                        ui.add(egui::Checkbox::new(&mut state.settings.monsters.immunities, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("normal_mobs"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.normal_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.normal_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("minions"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.minions_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.minions_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("champions"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.champions_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.champions_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("uniques"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.unique_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.unique_mob_color);
                        ui.end_row();

                        ui.label(localisation.get_primemh("bosses"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.monsters.boss_mobs_size)
                                .range(0.5..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.monsters.boss_mob_color);
                        ui.end_row();
                    });
            });

        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("lines_and_pathfinding"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("lines_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_waypoint_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.waypoint_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.waypoint_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_exit_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.exit_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.exit_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_quest_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.quest_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.quest_rgba);
                        ui.end_row();

                        ui.label(localisation.get_primemh("show_boss_lines"));
                        ui.add(egui::Checkbox::new(&mut state.settings.lines.boss_enabled, ""));
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.lines.boss_rgba);
                        ui.end_row();
                    });
            });
        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("item_log"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("my_grid2")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("item_show_log"));
                        ui.add(egui::Checkbox::new(&mut state.settings.item_log.enabled, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("item_log_text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.item_log.text_size)
                                .range(3.0..=60.0)
                                .speed(0.1),
                        );
                        ui.end_row();
                        ui.label(localisation.get_primemh("item_duration"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.item_log.text_duration)
                                .range(0..=90)
                                .speed(1),
                        );
                        ui.end_row();

                        ui.label(localisation.get_primemh("item_ground_alerts"));
                        ui.add(egui::Checkbox::new(&mut state.settings.item_log.ground_alerts, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("item_ground_text_size"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.item_log.ground_alerts_text_size)
                                .range(3.0..=60.0)
                                .speed(0.1),
                        );
                        ui.end_row();
                        ui.label(localisation.get_primemh("item_full_name"));
                        ui.add(egui::Checkbox::new(&mut state.settings.item_log.ground_alerts_show_suffix_prefix, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("item_tts"));
                        ui.add(egui::Checkbox::new(&mut state.settings.item_log.voice_enabled, ""));
                        ui.end_row();

                        ui.label(localisation.get_primemh("item_volume"));
                        ui.add(egui::Slider::new(&mut state.settings.item_log.voice_volume, 0..=100));
                        ui.end_row();

                        ui.label(localisation.get_primemh("item_speed"));
                        ui.add(egui::Slider::new(&mut state.settings.item_log.voice_speed, -5..=5));
                        ui.end_row();
                    });
            });
        ui.separator();
        egui::CollapsingHeader::new(localisation.get_primemh("missiles"))
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("missiles_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(localisation.get_primemh("show_missiles"));
                        ui.add(egui::Checkbox::new(&mut state.settings.missiles.enabled, ""));
                        ui.end_row();

                        // Fire
                        ui.label(localisation.get_primemh("fire"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.fire_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.fire_color);
                        ui.end_row();

                        // Cold
                        ui.label(localisation.get_primemh("cold"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.cold_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.cold_color);
                        ui.end_row();

                        // Poison
                        ui.label(localisation.get_primemh("poison"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.poison_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.poison_color);
                        ui.end_row();

                        // Lightning
                        ui.label(localisation.get_primemh("lightning"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.lightning_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.lightning_color);
                        ui.end_row();

                        // Physical
                        ui.label(localisation.get_primemh("physical"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.physical_size)
                                .range(0.1..=10.0)
                                .speed(0.1),
                        );
                        ui.color_edit_button_srgba_unmultiplied(&mut state.settings.missiles.physical_color);
                        ui.end_row();

                        // Magic
                        ui.label(localisation.get_primemh("magic"));
                        ui.add(
                            egui::DragValue::new(&mut state.settings.missiles.magic_size)
                                .range(0.1..=10.0)
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
        
        ui.label(splash_text.clone());

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(localisation.get_primemh("quit")).clicked() {
                app.exit();
            }
            if ui.button(localisation.get_primemh("save_settings")).clicked() {
                state.settings.save();
            }
        });
        
    });
}


pub fn create_language_select_ui(app: &mut App, ctx: &Context, state: &mut State) {
    setup_custom_fonts(ctx);
    ctx.set_pixels_per_point(app.window().dpi() as f32);
    egui::Window::new("Choose your language").default_open(true).collapsible(false).resizable(false).fixed_size((280.0, 280.0)).show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            let font_size = 20.0;
            if ui.button(egui::RichText::new("English").size(font_size)).clicked() { change_language(state, Locales::enUS)} 
            if ui.button(egui::RichText::new("Deutsch").size(font_size)).clicked() { change_language(state, Locales::deDE)}
            if ui.button(egui::RichText::new("한국어").size(font_size)).clicked() { change_language(state, Locales::koKR)}
            if ui.button(egui::RichText::new("臺灣話").size(font_size)).clicked() { change_language(state, Locales::zhTW)}
            if ui.button(egui::RichText::new("español").size(font_size)).clicked() { change_language(state, Locales::esES)}
            if ui.button(egui::RichText::new("français").size(font_size)).clicked() { change_language(state, Locales::frFR)}
            if ui.button(egui::RichText::new("italiano").size(font_size)).clicked() { change_language(state, Locales::itIT)}
            if ui.button(egui::RichText::new("polski").size(font_size)).clicked() { change_language(state, Locales::plPL)}
        });
    });
}

pub fn create_d2r_instance_picker_ui(app: &mut App, ctx: &Context, state: &mut State) {
    setup_custom_fonts(ctx);
    
    ctx.set_pixels_per_point(app.window().dpi() as f32);
    egui::Window::new("Choose your D2R instance").default_open(true).collapsible(false).resizable(false).fixed_size((640.0, 800.0)).show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            let font_size = 20.0;
            if ui.button(egui::RichText::new("English").size(font_size)).clicked() { change_language(state, Locales::enUS)} 
            if ui.button(egui::RichText::new("Deutsch").size(font_size)).clicked() { change_language(state, Locales::deDE)}
            if ui.button(egui::RichText::new("한국어").size(font_size)).clicked() { change_language(state, Locales::koKR)}
            if ui.button(egui::RichText::new("臺灣話").size(font_size)).clicked() { change_language(state, Locales::zhTW)}
            if ui.button(egui::RichText::new("español").size(font_size)).clicked() { change_language(state, Locales::esES)}
            if ui.button(egui::RichText::new("français").size(font_size)).clicked() { change_language(state, Locales::frFR)}
            if ui.button(egui::RichText::new("italiano").size(font_size)).clicked() { change_language(state, Locales::itIT)}
            if ui.button(egui::RichText::new("polski").size(font_size)).clicked() { change_language(state, Locales::plPL)}
        });
    });
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

fn change_language(state: &mut State, locale: Locales) {
    log::debug!("Selected new language {:?}", locale);
    state.settings.save_locale(locale);
    state.language_selector_visible = false;   
}
