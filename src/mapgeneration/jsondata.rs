use core::fmt;

use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use convert_case::Case;
use convert_case::Casing;

use super::mapimages::LevelImage;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SeedData {
    pub seed: u32,
    pub difficulty: u32,
    pub levels: Vec<LevelData>,
}

#[derive(Serialize, Default, Deserialize, Debug)]
pub struct LevelData {
    pub id: u32,
    #[serde(skip)]
    pub name: LevelName,
    pub offset: Offset,
    pub size: Size,
    pub objects: Vec<Object>,
    pub map: Vec<Vec<u64>>,
    #[serde(skip)]
    pub level_image: LevelImage,
}

impl LevelData {
    pub fn is_town(&self) -> bool {
        matches!(
            self.name,
            LevelName::RogueEncampment
                | LevelName::LutGholein
                | LevelName::KurastDocks
                | LevelName::PandemoniumFortress
                | LevelName::Harrogath
        )
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Offset {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object {
    pub id: u32,
    pub unit_type: String,
    pub x: u32,
    pub y: u32,
    #[serde(default)]
    pub name: String,
    pub category: String,
    #[serde(alias = "isGoodExit")]
    #[serde(default)]
    pub is_good_exit: bool,
    #[serde(default)]
    pub owned_level_id: u32,
}


impl Object {
    pub fn new_npc(x: u32, y: u32, id: u32) -> Object {
        Object {
            id,
            unit_type: String::from("NPC"),
            x,
            y,
            name: "".to_owned(),
            category: String::new(),
            is_good_exit: false,
            owned_level_id: 0,
        }
    }
    pub fn new_walkable_exit(x: u32, y: u32, owned_level_id: u32, exits: &[Object]) -> Object {
        let mut new_exit_id = 0;
        let attached_levels = crate::mapgeneration::walkableexits::get_attached_levels(owned_level_id);
        if !attached_levels.is_empty() {
            new_exit_id = attached_levels[0];
        }

        // if a neighbouring map has a matching exit then update the id of that exit as well
        let matching_exit: Vec<&Object> = exits
            .iter()
            .filter(|e| (e.x as i32 - x as i32).abs() < 3 && (e.y as i32 - y as i32).abs() < 3)
            .collect();
        if !matching_exit.is_empty() && matching_exit[0].owned_level_id > 0 {
            new_exit_id = matching_exit[0].owned_level_id
            // println!("2 level {} {} {}", owned_level_id, matching_exit[0].owned_level_id, matching_exit[0].id);
        }

        Object {
            id: new_exit_id,
            unit_type: String::from("Exit"),
            x,
            y,
            name: "".to_owned(),
            category: String::from("Exit"),
            is_good_exit: false,
            owned_level_id,
        }
    }
}

#[repr(u32)]
#[derive(FromPrimitive, Debug, Copy, Clone, Default, PartialEq)]
pub enum LevelName {
    #[default]
    None = 0,
    RogueEncampment,
    BloodMoor,
    ColdPlains,
    StonyField,
    DarkWood,
    BlackMarsh,
    TamoeHighland,
    DenOfEvil,
    CaveLevel1,
    UndergroundPassageLevel1,
    HoleLevel1,
    PitLevel1,
    CaveLevel2,
    UndergroundPassageLevel2,
    HoleLevel2,
    PitLevel2,
    BurialGrounds,
    Crypt,
    Mausoleum,
    ForgottenTower,
    TowerCellarLevel1,
    TowerCellarLevel2,
    TowerCellarLevel3,
    TowerCellarLevel4,
    TowerCellarLevel5,
    MonasteryGate,
    OuterCloister,
    Barracks,
    JailLevel1,
    JailLevel2,
    JailLevel3,
    InnerCloister,
    Cathedral,
    CatacombsLevel1,
    CatacombsLevel2,
    CatacombsLevel3,
    CatacombsLevel4,
    Tristram,
    SecretCowLevel,
    LutGholein,
    RockyWaste,
    DryHills,
    FarOasis,
    LostCity,
    ValleyOfSnakes,
    CanyonOfTheMagi,
    SewersLevel1,
    SewersLevel2,
    SewersLevel3,
    HaremLevel1,
    HaremLevel2,
    PalaceCellarLevel1,
    PalaceCellarLevel2,
    PalaceCellarLevel3,
    StonyTombLevel1,
    HallsOfTheDeadLevel1,
    HallsOfTheDeadLevel2,
    ClawViperTempleLevel1,
    StonyTombLevel2,
    HallsOfTheDeadLevel3,
    ClawViperTempleLevel2,
    MaggotLairLevel1,
    MaggotLairLevel2,
    MaggotLairLevel3,
    AncientTunnels,
    TalRashasTombStar,
    TalRashasTombSquare,
    TalRashasTombSemiCircle,
    TalRashasTombCircle,
    TalRashasTombTwoChevrons,
    TalRashasTombTriangle,
    TalRashasTombCirclewithline,
    DurielsLair,
    ArcaneSanctuary,
    KurastDocks,
    SpiderForest,
    GreatMarsh,
    FlayerJungle,
    LowerKurast,
    KurastBazaar,
    UpperKurast,
    KurastCauseway,
    Travincal,
    ArachnidLair,
    SpiderCavern,
    SwampyPitLevel1,
    SwampyPitLevel2,
    FlayerDungeonLevel1,
    FlayerDungeonLevel2,
    SwampyPitLevel3,
    FlayerDungeonLevel3,
    Act3SewersLevel1,
    Act3SewersLevel2,
    RuinedTemple,
    DisusedFane,
    ForgottenReliquary,
    ForgottenTemple,
    RuinedFane,
    DisusedReliquary,
    DuranceOfHateLevel1,
    DuranceOfHateLevel2,
    DuranceOfHateLevel3,
    PandemoniumFortress,
    OuterSteppes,
    PlainsOfDespair,
    CityOfTheDamned,
    RiverOfFlame,
    ChaosSanctuary,
    Harrogath,
    BloodyFoothills,
    FrigidHighlands,
    ArreatPlateau,
    CrystallinePassage,
    FrozenRiver,
    GlacialTrail,
    DrifterCavern,
    FrozenTundra,
    AncientsWay,
    IcyCellar,
    ArreatSummit,
    NihlathaksTemple,
    HallsOfAnguish,
    HallsOfPain,
    HallsOfVaught,
    Abaddon,
    PitOfAcheron,
    InfernalPit,
    WorldstoneKeepLevel1,
    WorldstoneKeepLevel2,
    WorldstoneKeepLevel3,
    ThroneOfDestruction,
    WorldstoneChamber,
    MatronsDen,
    ForgottenSands,
    FurnaceOfPain,
    UberTristram
}

impl fmt::Display for LevelName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = format!("{:?}", self).to_case(Case::Title).replace(" Of ", " of ").replace(" The ", " the ");
        write!(f, "{}", formatted)
    }
}
