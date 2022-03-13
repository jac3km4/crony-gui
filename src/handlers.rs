use phf::phf_map;

use crate::custom_handler;
use crate::host::CustomHandler;

#[inline]
pub fn get_custom_handler(name: &str) -> Option<&CustomHandler> {
    CUSTOM_HANDLERS.get(name)
}

// some functions accept extra arguments, they can be defined here
pub static CUSTOM_HANDLERS: phf::Map<&'static str, CustomHandler> = phf_map! {
    "GiveXP" => custom_handler!(i64),
    "GiveQuestXP" => custom_handler!(i64),
};
