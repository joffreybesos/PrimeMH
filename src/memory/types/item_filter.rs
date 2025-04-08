use crate::{ITEM_FILTER_FILE, LOCALISATION};
use regex::Regex;
use serde::{de, Deserialize, Deserializer};
use serde_yaml::Error;
use std::{collections::HashMap, fs::File, path::Path, str::FromStr};

use super::item::{BaseItem, ItemUnit, Quality};
use convert_case::{Case, Casing};

#[derive(Clone, PartialEq)]
pub struct ItemFilters {
    pub filters: HashMap<BaseItem, Vec<ItemFilter>>,
}

impl ItemFilters {
    pub fn load() -> Option<Self> {
        let localisation = LOCALISATION.lock().unwrap();
        let mut path = std::env::current_dir().unwrap();
        path.push(ITEM_FILTER_FILE);
        if let Ok(file) = File::open(path.as_path()) {
            let results: Result<HashMap<BaseItem, Vec<ItemFilter>>, Error> = serde_yaml::from_reader(file);
            if results.is_ok() {
                return Some(ItemFilters {
                    filters: results.unwrap(),
                });
            } else {
                log::error!("{}", format!("{}\n{}", localisation.get_primemh("error3"), results.err().unwrap()));
                None
            }
        } else {
            log::error!("{}", format!("{}\n{:?}", localisation.get_primemh("error4"), path));
            None
        }
    }

    pub fn match_filter(&self, item: &ItemUnit) -> (bool, Option<PlaySound>) {
        let filters = match self.filters.get(&item.txt_file_no) {
            None => return (false, None), // base item not in filter
            Some(filters) => filters,
        };

        if filters.is_empty() {
            // no filters of base item, so match immediately
            return (true, None);
        }

        for filter in filters.iter() {
            if (match &filter.ethereal {
                Some(eth) => &item.is_ethereal() == eth,
                None => true,
            } && match &filter.quality {
                Some(qualities) => qualities.contains(&item.quality),
                None => true,
            } && match &filter.sockets {
                Some(socket_vec) => socket_vec.contains(&item.num_sockets),
                None => true,
            }) {
                return (true, filter.play_sound_on_drop.clone())
            }
        }
        return (false, None)
    }
}

#[derive(Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ItemFilter {
    pub quality: Option<Vec<Quality>>,
    pub ethereal: Option<bool>,
    pub play_sound_on_drop: Option<PlaySound>,
    pub sockets: Option<Vec<u8>>,
    pub identified: Option<bool>,
    pub stats: Option<Vec<FilterStat>>,
}

impl<'de> Deserialize<'de> for Quality {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        let s = original.to_case(Case::Pascal);
        let quality = match Quality::from_str(&s) {
            Ok(quality) => quality,
            Err(e) => {
                return Err(de::Error::custom(format!("Invalid item quality : '{}' Error: '{}'", original, e)));
            }
        };
        Ok(quality)
    }
}

impl<'de> Deserialize<'de> for BaseItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        let s = original.to_case(Case::Pascal);
        let item: BaseItem = match BaseItem::from_str(&s) {
            Ok(item) => item,
            Err(e) => {
                return Err(de::Error::custom(format!("Invalid item name : '{}' Error: '{}'", original, e)));
            }
        };
        Ok(item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaySound {
    pub custom_sound_file: Option<String>
}

impl<'de> Deserialize<'de> for PlaySound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let filename = String::deserialize(deserializer)?;
        if filename == "true" {
            Ok(PlaySound { custom_sound_file: Some(String::from("default.mp3")) })
        } else if filename == "false" {
            Ok(PlaySound { custom_sound_file: None })
        } else {
            if Path::new(&filename).exists() {
                Ok(PlaySound { custom_sound_file: Some(filename) })
            } else {
                log::info!("ERROR: Sound file '{}' not found, using default sound", filename);
                Ok(PlaySound { custom_sound_file: Some(String::from("default.mp3")) })
            }
        }
    }
}



#[derive(Clone, PartialEq, Debug)]
pub struct FilterStat {
    pub stat: String,
    pub operator: String,
    pub value: i16,
}

impl<'de> Deserialize<'de> for FilterStat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let original: String = String::deserialize(deserializer)?;
        let re = Regex::new(r"^(.*?)\s*([<>])\s*(\d+)$").unwrap();

        match re.captures(&original) {
            Some(caps) => {
                // Extract the stat, operator, and value from the regex captures
                let stat = caps.get(1).unwrap().as_str().trim();
                let operator = caps.get(2).unwrap().as_str();
                let value: i16 = caps.get(3).unwrap().as_str().parse().unwrap();

                log::info!("Stat: '{}', Operator: '{}', Value: {}", stat, operator, value);
                return Ok(FilterStat {
                    stat: stat.to_string(),
                    operator: operator.to_string(),
                    value,
                });
            },
            None => {
                return Err(de::Error::custom(format!("Invalid item stat filter : '{}'", original)));
            }
        }
    }
}

pub enum FilterStats {
    Defense,
    Durability,
    Level,
    Strength,
    Dexterity,
    Energy,
    Vitality,
    RequiredLevel,
    RequiredStrength,
    RequiredDexterity,
    RequiredEnergy,
    RequiredVitality,
    EnhancedDefense,
    EnhancedDamage,
    EnhancedDamagePercent,
}