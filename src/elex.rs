use std::borrow::Cow;
use std::{mem, ptr};

use egui_hook::import_foreign;
use pelite::pe64::{Pe, PeView};

pub fn get_all_functions<'a>() -> impl Iterator<Item = (Cow<'a, str>, FunctionPtr)> {
    unsafe { &*get_function_registry() }
        .functions()
        .map(|fun| (fun.name(), fun.ptr()))
}

#[inline]
pub fn get_player() -> Entity {
    Entity(get_player_ptr())
}

#[inline]
pub fn get_player_look_at() -> Entity {
    let player = get_player();
    let mut entity = Entity(ptr::null());
    get_player_look_at_ptr(&player, &mut entity);
    entity
}

import_foreign!(0x0867080, get_function_registry() -> *const FunctionRegistry);
import_foreign!(0x040B710, get_player_ptr() -> *const GameObject);
import_foreign!(0x0B17FF0, get_player_look_at_ptr(player: *const Entity, entity: *mut Entity) -> ());

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entity(*const GameObject);

impl Entity {
    #[inline]
    pub fn null() -> Entity {
        Entity(std::ptr::null())
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FunctionRegistry {
    list1: FunctionList,
    list2: FunctionList,
    list3: FunctionList,
    list4: FunctionList,
}

impl FunctionRegistry {
    // TODO: figure out how to make use of list1 and list2
    pub fn functions(&self) -> impl Iterator<Item = &FunctionDef> {
        self.list3.as_slice().iter().chain(self.list4.as_slice())
    }
}

#[derive(Debug)]
#[repr(C)]
struct FunctionList {
    items: *const FunctionDef,
    count: u32,
}

impl FunctionList {
    #[inline]
    pub fn as_slice(&self) -> &[FunctionDef] {
        unsafe { std::slice::from_raw_parts(self.items, self.count as usize) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FunctionDef {
    name: *const i8,
    unk1: i64,
    ptr: usize,
    unk2: i64,
}

impl FunctionDef {
    #[inline]
    pub fn name<'a>(&self) -> Cow<'a, str> {
        unsafe { std::ffi::CStr::from_ptr(self.name) }.to_string_lossy()
    }

    #[inline]
    pub fn ptr(&self) -> FunctionPtr {
        FunctionPtr(self.ptr)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionPtr(usize);

impl FunctionPtr {
    #[inline]
    pub fn invoke_default<A>(&self, val: A) -> i64 {
        let player = get_player();
        let func: extern "C" fn(Entity, Entity, A) -> i64 = unsafe { mem::transmute(self.0) };
        func(player, player, val)
    }

    #[inline]
    pub fn invoke_with<A>(&self, a: Entity, b: Entity, val: A) -> i64 {
        let func: extern "C" fn(Entity, Entity, A) -> i64 = unsafe { mem::transmute(self.0) };
        func(a, b, val)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Str {
    ptr: *const char,
    len: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct RuntimeProperty {
    vft: *const (),
    name: Str,
    name_override: Str,
    next: *const RuntimeProperty,
    parent: *const Class,
    typ: *const Type,
    flags: u32,
    name_hash: u32,
    name_override_hash: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct MemberProperty {
    prop: RuntimeProperty,
    short_name: Str,
    offset: u32,
    unk2: i32,
}

// TODO: reverse
#[repr(C)]
pub struct Class;

// TODO: reverse
#[repr(C)]
pub struct Type;

// TODO: reverse
#[repr(C)]
pub struct GameObject;

const SUPPORTED_VERSION_TS: u32 = 1646857151;

pub fn version_check() -> bool {
    let handle = unsafe { egui_hook::GetModuleHandleA(egui_hook::PSTR(ptr::null())) };
    let pe = unsafe { PeView::module(handle.0 as *const _) };
    let pe_ts = pe.file_header().TimeDateStamp;
    SUPPORTED_VERSION_TS == pe_ts
}
