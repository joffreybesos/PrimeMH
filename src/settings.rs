#![allow(non_camel_case_types)]
use config::{Config, ConfigError, File};
use serde::{Deserialize, Deserializer, Serialize};
use strum::EnumString;
use std::{env, fs, path::PathBuf};
use locale_config::Locale;
use std::str::FromStr;


use crate::SETTINGS_FILE;

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct ItemToolTip {
    pub enabled: bool,
    pub text_size: f32,
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
}

impl Default for Lines {
    fn default() -> Self {
        Lines {
            waypoint_enabled: true,
            waypoint_path_enabled: true,
            waypoint_rgba: [255, 255, 0, 127],
            exit_enabled: true,
            exit_path_enabled: true,
            exit_rgba: [255, 0, 255, 127],
            quest_enabled: true,
            quest_path_enabled: true,
            quest_rgba: [0, 255, 0, 127],
            boss_enabled: true,
            boss_path_enabled: true,
            boss_rgba: [255, 0, 0, 127],
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

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Chests {
    pub enabled: bool,
    pub size: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Portals {
    pub enabled: bool,
    pub size: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Shrines {
    pub enabled: bool,
    pub size: f32,
    pub text_size: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Monsters {
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

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct General {
    pub debug: bool,
    pub d2lodpath: PathBuf,
    pub blacha_exe: PathBuf,
    pub render_scale: f32,
    pub fps_limit: u8,
    pub title: String,
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
}

fn get_true() -> bool {
    true
}

fn get_eight() -> u8 {
    8
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub general: General,
    pub visual: Visual,
    pub missiles: Missiles,
    pub chests: Chests,
    pub portals: Portals,
    pub shrines: Shrines,
    pub lines: Lines,
    pub monsters: Monsters,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut path = env::current_dir().unwrap();
        path.push(SETTINGS_FILE);

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
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum MapPosition {
    #[default]
    Center,
    TopLeft,
    TopRight,
}


#[derive(Debug, Serialize, Default, EnumString, PartialEq, Eq)]
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
