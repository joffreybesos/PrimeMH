use super::{process::D2RInstance, structs::{MenuStates, UnitHashTable}};
use crate::types::{
     get_missiles, get_units, item::ItemUnit, keybindings::KeyBindings, last_hovered::LastHovered, missile::MissileUnit, npc::NPCUnit, object::GameObjectUnit, player::{self, PlayerUnit}, roster::{self, RosterItem}, seedvalues::SeedValues
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct GameData {
    pub seed_values: SeedValues,
    pub player: PlayerUnit,
    pub players: Vec<PlayerUnit>,
    pub npcs: Vec<NPCUnit>,
    pub roster_items: Vec<RosterItem>,
    pub objects: Vec<GameObjectUnit>,
    pub items: Vec<ItemUnit>,
    pub missiles: Vec<MissileUnit>,
    pub menus: MenuStates,
    pub last_hovered: LastHovered,
    pub keybindings: KeyBindings,
}

impl GameData {
    pub fn read_game_memory(d2rprocess: &mut D2RInstance) -> Option<GameData> {
        // let offsets = d2rprocess.offsets.clone();
        if d2rprocess.offsets.unit_table == 0 {
            d2rprocess.refresh_offsets();
        }
        if d2rprocess.offsets.unit_table == 0 {
            return None;
        }
        let unit_ptrs: UnitHashTable = d2rprocess.read_mem_offset::<UnitHashTable>(d2rprocess.offsets.unit_table);
        let players: Vec<PlayerUnit> = get_units(d2rprocess, unit_ptrs.player_ptrs);
        if players.is_empty() {
            // not in a current game
            // println!("No players");
            return None;
        }
        let roster_items: Vec<RosterItem> = roster::get_roster(d2rprocess);
        let first_roster = roster_items.get(0);
        let player_roster = match first_roster {
            Some(player_roster) => player_roster,
            None => return None,
        };
        let player: &PlayerUnit = match player::get_current_player(&players, player_roster.unit_id) {
            Some(p) => p,
            None => {
                // println!("{} No player found", players.len());
                return None;
            }
        };

        let seed_values = SeedValues::new(d2rprocess, player);
        if seed_values.dw_init_seed_hash == 0 {
            return None;
            
        }
        let npcs: Vec<NPCUnit> = get_units(d2rprocess, unit_ptrs.npc_ptrs);
        let objects: Vec<GameObjectUnit> = get_units(d2rprocess, unit_ptrs.object_ptrs);
        let items: Vec<ItemUnit> = get_units(d2rprocess, unit_ptrs.item_ptrs);
        let missiles: Vec<MissileUnit> = get_missiles(d2rprocess, unit_ptrs.missile_ptrs, unit_ptrs.server_missile_ptrs, player);

        // let panels = get_panels(d2rprocess);

        let menus: MenuStates = MenuStates::get_menu_states(d2rprocess);
        let last_hovered: LastHovered = LastHovered::get_hovered(d2rprocess);

        let keybindings: KeyBindings = KeyBindings::new(d2rprocess);
        

        Some(GameData {
            seed_values,
            player: player.clone(),
            players,
            npcs,
            roster_items,
            objects,
            items,
            missiles,
            menus,
            last_hovered,
            keybindings,
        })
    }

    #[allow(unused)]
    pub fn is_in_town(&self) -> bool {
        match self.seed_values.level {
            1 | 40 | 75 | 103 | 109 => true,
            _ => false
        }
    }
}


pub fn get_last_game_name(d2rprocess: &D2RInstance) -> String {
    let last_game_arr = d2rprocess.read_mem_offset::<[u8; 15]>(d2rprocess.offsets.last_game_name);
    let mut game_name = d2rprocess.parse_arr_to_string(&last_game_arr);

    // there's a problem with non-english characters sometimes
    if !game_name.chars().all(|x| x.is_ascii()) {
        game_name = "".to_string();
    }
    game_name
}
