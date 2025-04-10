#![allow(non_camel_case_types)]
use config::{Config, ConfigError, File};
use device_query::Keycode;
use crate::gui::hotkeys::HotKey;
use serde::{Deserialize, Deserializer, Serialize};
use strum::EnumString;
use std::{env, fs, path::PathBuf};
use locale_config::Locale;
use std::str::FromStr;


use crate::SETTINGS_FILE;

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct ItemToolTip {
    #[serde(default = "get_true")]
    pub enabled: bool,
    #[serde(default = "get_four")]
    pub text_size: f32,
}

impl Default for ItemToolTip {
    fn default() -> Self {
        ItemToolTip {
            enabled: true,
            text_size: 4.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Lines {
    pub waypoint_enabled: bool,
    pub waypoint_path_enabled: bool,
    pub waypoint_rgba: [u8; 4],
    pub exit_enabled: bool,
    pub exit_path_enabled: bool,
    pub exit_rgba: [u8; 4],
    pub quest_enabled: bool,
    pub quest_path_enabled: bool,
    pub quest_rgba: [u8; 4],
    pub boss_enabled: bool,
    pub boss_path_enabled: bool,
    pub boss_rgba: [u8; 4],
    #[serde(default = "get_1")]
    pub line_width: f32,
}

impl Default for Lines {
    fn default() -> Self {
        Lines {
            waypoint_enabled: true,
            waypoint_path_enabled: false,
            waypoint_rgba: [255, 255, 0, 127],
            exit_enabled: true,
            exit_path_enabled: false,
            exit_rgba: [255, 0, 255, 127],
            quest_enabled: true,
            quest_path_enabled: false,
            quest_rgba: [0, 255, 0, 127],
            boss_enabled: true,
            boss_path_enabled: false,
            boss_rgba: [255, 0, 0, 127],
            line_width: 1.0
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct BuffBar {
    pub enabled: bool,
    pub vertical_pos: f32,
    #[serde(default = "get_half")]
    pub horizontal_pos: f32,
    pub icon_scale: f32,
    #[serde(default = "get_20")]
    pub charges_font_size: f32,
    #[serde(default = "get_30")]
    pub timer_font_size: f32,
}

impl Default for BuffBar {
    fn default() -> Self {
        BuffBar {
            enabled: true,
            vertical_pos: 0.75,
            horizontal_pos: 0.5,
            icon_scale: 18.0,
            charges_font_size: 30.0,
            timer_font_size: 20.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct PartyInfo {
    pub enabled: bool,
    
}

impl Default for PartyInfo {
    fn default() -> Self {
        PartyInfo {
            enabled: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Visual {
    pub scale: f32,
    pub map_opacity: f32,
    pub always_show_map: bool,
    pub hide_map_menus_open: bool,
    #[serde(default = "get_four")]
    pub exit_label_text_size: f32,
}

impl Default for Visual {
    fn default() -> Self {
        Visual {
            scale: 3.3,
            map_opacity: 0.65,
            always_show_map: true,
            hide_map_menus_open: true,
            exit_label_text_size: 4.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct ItemLog {
    pub enabled: bool,
    pub text_size: f32,
    pub text_duration: i32,
    pub ground_alerts: bool,
    pub ground_alerts_text_size: f32,
    #[serde(default = "get_true")]
    pub sound_enabled: bool,
    pub voice_enabled: bool,
    pub voice_volume: u32,
    pub voice_speed: i32,
    pub ground_alerts_show_suffix_prefix: bool,
}

impl Default for ItemLog {
    fn default() -> Self {
        ItemLog {
            enabled: true,
            text_size: 16.0,
            text_duration: 30,
            ground_alerts: true,
            ground_alerts_text_size: 4.5,
            sound_enabled: true,
            voice_enabled: true,
            voice_volume: 80,
            voice_speed: 2,
            ground_alerts_show_suffix_prefix: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Missiles {
    pub enabled: bool,
    pub fire_size: f32,
    pub fire_color: [u8; 4],
    pub cold_size: f32,
    pub cold_color: [u8; 4],
    pub poison_size: f32,
    pub poison_color: [u8; 4],
    pub lightning_size: f32,
    pub lightning_color: [u8; 4],
    pub physical_size: f32,
    pub physical_color: [u8; 4],
    pub magic_size: f32,
    pub magic_color: [u8; 4],
}

impl Default for Missiles {
    fn default() -> Self {
        Missiles {
            enabled: true,
            fire_size: 3.0,
            fire_color: [255, 0, 0, 127],
            cold_size: 3.0,
            cold_color:  [0, 208, 255, 127],
            poison_size: 3.0,
            poison_color: [50, 205, 50, 127],
            lightning_size: 3.0,
            lightning_color: [255, 255, 0, 70],
            physical_size: 3.0,
            physical_color: [205, 133, 63, 127],
            magic_size: 3.0,
            magic_color: [255, 136, 0, 127],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Chests {
    pub enabled: bool,
    pub size: f32,
}

impl Default for Chests {
    fn default() -> Self {
        Chests {
            enabled: true,
            size: 0.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Portals {
    pub enabled: bool,
    pub size: f32,
}

impl Default for Portals {
    fn default() -> Self {
        Portals {
            enabled: true,
            size: 2.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Shrines {
    pub enabled: bool,
    pub size: f32,
    pub text_size: f32,
}

impl Default for Shrines {
    fn default() -> Self {
        Shrines {
            enabled: true,
            size: 0.1,
            text_size: 5.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Monsters {
    #[serde(default = "get_true")]
    pub enabled: bool,
    pub immunities: bool,
    pub normal_mobs: bool,
    pub normal_mobs_size: f32,
    pub normal_mob_color: [u8; 4],
    pub minions_mobs: bool,
    pub minions_mobs_size: f32,
    pub minions_mob_color: [u8; 4],
    pub champions_mobs: bool,
    pub champions_mobs_size: f32,
    pub champions_mob_color: [u8; 4],
    pub unique_mobs: bool,
    pub unique_mobs_size: f32,
    pub unique_mob_color: [u8; 4],
    pub boss_mobs: bool,
    pub boss_mobs_size: f32,
    pub boss_mob_color: [u8; 4],
}

impl Default for Monsters {
    fn default() -> Self {
        Monsters {
            enabled: true,
            immunities: true,
            normal_mobs: true,
            normal_mobs_size: 1.5,
            normal_mob_color: [255, 255, 255, 255],
            minions_mobs: true,
            minions_mobs_size: 1.5,
            minions_mob_color: [255, 255, 255, 255],
            champions_mobs: true,
            champions_mobs_size: 3.0,
            champions_mob_color: [96, 92, 216, 255],
            unique_mobs: true,
            unique_mobs_size: 3.0,
            unique_mob_color: [212, 175, 55, 255],
            boss_mobs: true,
            boss_mobs_size: 3.0,
            boss_mob_color: [255, 0, 0, 255],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct HotKeys {
    #[serde(default = "default_pagedown_hotkey")]
    pub hotkey_toggle_map: HotKey,
    #[serde(default = "default_home_hotkey")]
    pub hotkey_toggle_menu: HotKey,
    #[serde(default = "default_exit_hotkey")]
    pub hotkey_exit: HotKey,
}

impl Default for HotKeys {
    fn default() -> Self {
        HotKeys {
            hotkey_toggle_map: default_pagedown_hotkey(),
            hotkey_toggle_menu: default_home_hotkey(),
            hotkey_exit: default_exit_hotkey(),
        }
    }
}

fn default_pagedown_hotkey() -> HotKey {
    HotKey::new(Keycode::PageDown, false)
}

fn default_home_hotkey() -> HotKey {
    HotKey::new(Keycode::Home, false)
}

fn default_exit_hotkey() -> HotKey {
    HotKey::new(Keycode::End, true)
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct General {
    pub debug: bool,
    pub d2lodpath: PathBuf,
    pub blacha_exe: PathBuf,
    pub render_scale: f32,
    pub fps_limit: u8,
    pub map_position: MapPosition,
    #[serde(default = "get_eight")]
    pub multisampling: u8,
    #[serde(default = "get_true")]
    pub vsync: bool,
    #[serde(default = "get_true")]
    pub high_dpi: bool,
    #[serde(default = "get_true")]
    pub overlay_mode: bool,
    #[serde(default)]
    pub language: Locales,
    #[serde(default)]
    pub disable_log: bool,
}

impl Default for General {
    fn default() -> Self {
        General {
            debug: false,
            d2lodpath: PathBuf::from("bin/d2lod"),
            blacha_exe: PathBuf::from("bin/d2-mapgen.exe"),
            render_scale: 1.0,
            fps_limit: 60,
            map_position: MapPosition::Center,
            multisampling: 8,
            vsync: true,
            high_dpi: true,
            overlay_mode: true,
            language: Locales::enUS,
            disable_log: false,
        }
    }
}

fn get_true() -> bool {
    true
}

fn get_eight() -> u8 {
    8
}

fn get_four() -> f32 {
    4.0
}

fn get_half() -> f32 {
    0.5
}

fn get_20() -> f32 {
    20.0
}

fn get_30() -> f32 {
    30.0
}

fn get_1() -> f32 {
    1.0
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Settings {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub visual: Visual,
    #[serde(default)]
    pub item_log: ItemLog,
    #[serde(default)]
    pub missiles: Missiles,
    pub chests: Chests,
    #[serde(default)]
    pub portals: Portals,
    #[serde(default)]
    pub shrines: Shrines,
    #[serde(default)]
    pub lines: Lines,
    #[serde(default)]
    pub monsters: Monsters,
    #[serde(default)]
    pub buffbar: BuffBar,
    #[serde(default)]
    pub item_hover: ItemToolTip,
    #[serde(default)]
    pub hotkeys: HotKeys,
    #[serde(default)]
    pub party_info: PartyInfo,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            general: General::default(),
            visual: Visual::default(),
            item_log: ItemLog::default(),
            missiles: Missiles::default(),
            chests: Chests::default(),
            portals: Portals::default(),
            shrines: Shrines::default(),
            lines: Lines::default(),
            monsters: Monsters::default(),
            buffbar: BuffBar::default(),
            item_hover: ItemToolTip::default(),
            hotkeys: HotKeys::default(),
            party_info: PartyInfo::default(),
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut path = env::current_dir().unwrap();
        path.push(SETTINGS_FILE);

        if !path.exists() {
            let s = Settings::default();
            s.save();
            return Ok(s);
        }

        let s = Config::builder().add_source(File::with_name(SETTINGS_FILE)).build()?;
        s.try_deserialize()
    }

    pub fn save(&self) {
        let mut path = env::current_dir().unwrap();
        // path.push("src");
        path.push(SETTINGS_FILE);

        let toml_string = toml::to_string(&self).expect("Could not encode settings.toml file!");
        fs::write(path, toml_string).expect("Could not write to settings.toml file!");
    }

    pub fn detect_locale(&mut self) {
        let locale = Locale::current();
        log::info!("System's detected locale: {}", locale);
        if self.general.language == Locales::Unknown {
            let locale = locale.to_string().replace("-", "");
            self.general.language = Locales::from_str(locale.as_str()).unwrap_or(Locales::enUS);
            
        }
        log::info!("PrimeMH configured locale: {:?}", self.general.language);
        self.save();
    }

    pub fn save_locale(&mut self, locale: Locales) {
        log::info!("Saving locale: {:?}", locale);
        
        self.general.language = locale;
        self.save();
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum MapPosition {
    #[default]
    Center,
    TopLeft,
    TopRight,
}


#[derive(Debug, Serialize, Default, EnumString, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Locales {
    enUS,
    zhTW,
    deDE,
    esES,
    frFR,
    itIT,
    koKR,
    plPL,
    enBG,
    #[default]
    Unknown,
}


impl<'de> Deserialize<'de> for Locales {
    fn deserialize<D>(deserializer: D) -> Result<Locales, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        // Check for alternative values and map them to enum variants
        match s.as_str() {
            "enUS" => Ok(Locales::enUS),
            "zhTW" => Ok(Locales::zhTW),
            "deDE" => Ok(Locales::deDE),
            "esES" => Ok(Locales::esES),
            "frFR" => Ok(Locales::frFR),
            "itIT" => Ok(Locales::itIT),
            "koKR" => Ok(Locales::koKR),
            "plPL" => Ok(Locales::plPL),
            "enBG" => Ok(Locales::enBG),
            "en" => Ok(Locales::enUS),
            "bg" => Ok(Locales::enBG),
            "es" => Ok(Locales::esES),
            "ko" => Ok(Locales::koKR),
            "nl" => Ok(Locales::enUS),
            "de" => Ok(Locales::deDE),
            "it" => Ok(Locales::itIT),
            "fr" => Ok(Locales::frFR),
            "tw" => Ok(Locales::zhTW),
            _ => Ok(Locales::Unknown), // Default to Unknown if value doesn't match any variants
        }
    }
}
