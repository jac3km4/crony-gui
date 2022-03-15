use std::any::TypeId;
use std::fmt::Debug;
use std::sync::mpsc;

use heck::ToSnakeCase;
use rhai::plugin::CallableFunction;
use rhai::{Dynamic, Engine, FnAccess, FnNamespace, Module, RegisterNativeFunction, Scope};

use crate::{elex, handlers};

#[derive(Debug)]
pub struct ScriptHost {
    pub(crate) cmd: String,
    pub(crate) history: String,
    log_receiver: mpsc::Receiver<String>,
    is_active: bool,
    engine: Engine,
    scope: Scope<'static>,
}

impl Default for ScriptHost {
    fn default() -> Self {
        let mut engine = Engine::new();
        let (tx, tr) = mpsc::channel();

        engine.register_static_module("game", ScriptHost::create_game_module().into());
        engine.register_static_module("entity", ScriptHost::create_entity_module().into());

        engine.register_fn("log", move |val: Dynamic| {
            tx.send(val.to_string()).ok();
        });

        Self {
            cmd: String::new(),
            history: String::new(),
            log_receiver: tr,
            is_active: false,
            engine,
            scope: Scope::new(),
        }
    }
}

impl ScriptHost {
    pub fn process_command(&mut self) {
        self.history.push_str(&self.cmd);
        self.history.push('\n');

        if let Err(err) = self.engine.run_with_scope(&mut self.scope, &self.cmd) {
            self.history.push_str(&err.to_string());
            self.history.push('\n');
        }
        self.cmd.clear();
    }

    pub fn process_events(&mut self) {
        while let Ok(str) = self.log_receiver.try_recv() {
            self.history.push_str(&str);
            self.history.push('\n');
        }
    }

    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    fn create_game_module() -> Module {
        let mut module = Module::new();

        for (name, ptr) in elex::get_all_functions() {
            let name = name.to_snake_case();
            if let Some(custom_handler) = handlers::get_custom_handler(&name) {
                custom_handler(ptr).add(&name, &mut module);
            } else {
                module.set_native_fn(&name, move || Ok(ptr.invoke_default(())));
            }
        }
        module
    }

    fn create_entity_module() -> Module {
        let mut module = Module::new();
        module.set_native_fn("get_player", || Ok(elex::get_player()));
        module.set_native_fn("get_look_at", || Ok(elex::get_player_look_at()));
        module.set_native_fn("none", || Ok(elex::Entity::null()));
        module
    }
}

pub struct ExportedFunction(CallableFunction, Box<[TypeId]>);

impl ExportedFunction {
    #[inline]
    pub fn new<A: RegisterNativeFunction<Args, Ret>, Args, Ret>(val: A) -> Self {
        Self(val.into_callable_function(), A::param_types())
    }

    #[inline]
    fn add(self, name: &str, module: &mut Module) {
        module.set_fn(
            name,
            FnNamespace::Internal,
            FnAccess::Public,
            None,
            self.1,
            self.0,
        );
    }
}

#[macro_export]
macro_rules! custom_handler {
    ($expr:expr) => {
        |ptr| $crate::host::ExportedFunction::new($expr(ptr))
    };
}

#[macro_export]
macro_rules! type_handler {
    ($ty:ty) => {
        custom_handler! { |ptr: $crate::elex::FunctionPtr|
            move |arg: $ty| ptr.invoke_default(arg)
        }
    };
}
