use std::borrow::Cow;
use std::mem::transmute;

use egui_hook::import_foreign;

pub fn get_all_functions<'a>() -> impl Iterator<Item = (Cow<'a, str>, FunctionPtr)> {
    unsafe { &*get_function_registry() }
        .functions()
        .map(|fun| (fun.name(), fun.ptr()))
}

import_foreign!(0x040B710, get_player() -> *const Player);
import_foreign!(0x0867080, get_function_registry() -> *const FunctionRegistry);

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
    pub fn name<'a>(&self) -> Cow<'a, str> {
        unsafe { std::ffi::CStr::from_ptr(self.name) }.to_string_lossy()
    }

    pub fn ptr(&self) -> FunctionPtr {
        FunctionPtr(self.ptr)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionPtr(usize);

impl FunctionPtr {
    pub fn invoke_default<A>(&self, val: A) -> i64 {
        let ptr = get_player();
        let func: extern "C" fn(*const Player, *const Player, A) -> i64 =
            unsafe { transmute(self.0) };
        func(ptr, ptr, val)
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
pub struct Player;
