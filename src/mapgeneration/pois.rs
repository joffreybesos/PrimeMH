use crate::mapgeneration::jsondata::{LevelData, Object};

use super::jsondata::{LevelName, Offset};

use num_traits::FromPrimitive;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub struct POI {
    pub id: u32,
    pub poi_type: POIType,
    pub pos_x: f32,
    pub pos_y: f32,
    pub world_x: u32,
    pub world_y: u32,
    pub class: String,
    pub label: String,
}

impl Default for POI {
    fn default() -> Self {
        POI {
            id: 0,
            poi_type: POIType::default(),
            pos_x: 0.0,
            pos_y: 0.0,
            world_x: 0,
            world_y: 0,
            class: String::new(),
            label: String::new(),
        }
    }
}

impl POI {
    pub fn new(obj: &Object, offset: &Offset, poi_type: POIType) -> Self {
        Self::new_label(obj, offset, poi_type, String::new())
    }
    pub fn new_label(obj: &Object, offset: &Offset, poi_type: POIType, label: String) -> Self {
        POI {
            id: obj.id,
            poi_type,
            pos_x: obj.x as f32,
            pos_y: obj.y as f32,
            world_x: obj.x + offset.x,
            world_y: obj.y + offset.y,
            class: format!("{:?}", obj.category),
            label,
        }
    }
    pub fn new_shrine(x: u32, y: u32, offset: &Offset, label: String) -> Self {
        POI {
            id: 2,
            poi_type: POIType::Shrine,
            pos_x: x as f32 - offset.x as f32,
            pos_y: y as f32 - offset.y as f32,
            world_x: x,
            world_y: y,
            class: "".to_string(),
            label,
        }
    }
    pub fn new_well(x: u32, y: u32, offset: &Offset, label: String) -> Self {
        POI {
            id: 2,
            poi_type: POIType::Well,
            pos_x: x as f32 - offset.x as f32,
            pos_y: y as f32 - offset.y as f32,
            world_x: x,
            world_y: y,
            class: "".to_string(),
            label,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum POIType {
    Waypoint,
    Shrine,
    Well,
    Chest,
    SuperChest,
    Exit,
    GoodExit,
    QuestItem,
    NPCSpawn,
    #[default]
    Unknown,
}

// convert the blacha JSON to usable POI objects
pub fn get_preset_pois(level_data: &LevelData) -> Vec<POI> {
    let mut pois: Vec<POI> = vec![];

    for object in &level_data.objects {
        if object.category == "Waypoint" {
            pois.push(POI::new(object, &level_data.offset, POIType::Waypoint));
        }

        if object.category == "Lootable" && (level_data.id == 84 || level_data.id == 85 || level_data.id == 91) {
            pois.push(POI::new(object, &level_data.offset, POIType::SuperChest));
        }
        if object.category == "Lootable" && (level_data.id == 85 || level_data.id == 91) {
            pois.push(POI::new(object, &level_data.offset, POIType::QuestItem));
        }
        if object.category == "Super Chest" && object.id == 580 {
            pois.push(POI::new(object, &level_data.offset, POIType::SuperChest));
        }
        if object.category == "Shrine" {
            pois.push(POI::new(object, &level_data.offset, POIType::Shrine));
        }

        if object.category == "Well" {
            pois.push(POI::new(object, &level_data.offset, POIType::Well));
        }

        if object.unit_type == "Exit" {
            if object.is_good_exit && level_data.id == 46 {
                pois.push(POI::new(object, &level_data.offset, POIType::GoodExit));
            } else {
                let label = LevelName::from_u32(object.id).unwrap_or_default().to_string();
                let exit_poi = POI::new_label(object, &level_data.offset, POIType::Exit, label);
                pois.push(exit_poi);
            }
        }

        if object.unit_type == "NPC" {
            // summoner
            if level_data.id == 74 && object.id == 250 {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // izual
            if level_data.id == 105 && object.unit_type == "NPC" {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // maggot lair 3
            if level_data.id == 64 && object.unit_type == "NPC" {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // radament
            if level_data.id == 49 && object.id == 744 {
                pois.push(POI::new(object, &level_data.offset, POIType::NPCSpawn));
            }
            // nihlithak is calculated by the preset NPC on the _opposite_ side of the map
            if level_data.id == 124 && object.unit_type == "NPC"{
                let mut x = object.x;
                let mut y = object.y;
                if x == 30 && y == 208 {
                    // bottom right
                    x = 395;
                    y = 210;
                }
                if x == 206 && y == 32 {
                    // bottom left
                    x = 210;
                    y = 395;
                }
                if x == 207 && y == 393 {
                    // top right
                    x = 210;
                    y = 25;
                }
                if x == 388 && y == 216 {
                    //top left
                    x = 25;
                    y = 210;
                }
                let nihl_x = x as u32;
                let nihl_y = y as u32;

                let nihl = Object::new_npc(nihl_x, nihl_y, object.id);

                pois.push(POI::new(&nihl, &level_data.offset, POIType::NPCSpawn));
            }
        }

        // quest items
        if level_data.id != 75 {
            match object.name.as_str() {
                "orifice" | "gidbinn altar" | "Hellforge" | "cagedwussie1" | "Tome" | "LamTome" | "Inifuss"
                | "taintedsunaltar" | "Seal" | "StoneLambda" => {
                    pois.push(POI::new(object, &level_data.offset, POIType::QuestItem));
                }
                _ => (),
            }
        }
        //anya
        if object.unit_type == "NPC"&& level_data.id == 114 {
            pois.push(POI::new(object, &level_data.offset, POIType::QuestItem));
        }
    }
    pois
}
