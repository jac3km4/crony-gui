use phf::phf_map;

use crate::elex::{Entity, FunctionPtr};
use crate::host::ExportedFunction;
use crate::{custom_handler, type_handler};

pub type CustomHandler = fn(FunctionPtr) -> ExportedFunction;

#[inline]
pub fn get_custom_handler(name: &str) -> Option<&CustomHandler> {
    CUSTOM_HANDLERS.get(name)
}

// some functions accept extra arguments, they can be defined here
static CUSTOM_HANDLERS: phf::Map<&'static str, CustomHandler> = phf_map! {
    "advance_time" => type_handler!(i64),
    "give_quest_xp" => type_handler!(i64),
    "give_xp" => type_handler!(i64),
    "on_info_advance_playing_time_by_hours" => type_handler!(i64),
    "set_player_rank" => type_handler!(i64),
    "set_target_hour" => type_handler!(i64),

    "auto_loot" => custom_handler! { |ptr: FunctionPtr|
        move |looter: Entity, target: Entity| ptr.invoke_with(looter, target, ())
    },
    "kill" => custom_handler! { |ptr: FunctionPtr|
        move |instigator: Entity, target: Entity| ptr.invoke_with(instigator, target, ())
    },
    "join_player_party" => custom_handler! { |ptr: FunctionPtr|
        move |entity: Entity| ptr.invoke_with(entity, Entity::null(), ())
    },
    "dismiss_player_party" => custom_handler! { |ptr: FunctionPtr|
        move |entity: Entity| ptr.invoke_with(entity, Entity::null(), ())
    },
    "spawn_new_entity" => custom_handler! { |ptr: FunctionPtr|
        move |pos: Entity, entity: Entity| ptr.invoke_with(pos, entity, ())
    },
    "remove_npc" => custom_handler! { |ptr: FunctionPtr|
        move |npc: Entity| ptr.invoke_with(npc, Entity::null(), ())
    },
    "place_summoned_party_member" => custom_handler! { |ptr: FunctionPtr|
        move |leader: Entity, member: Entity| ptr.invoke_with(leader, member, ())
    },
};
