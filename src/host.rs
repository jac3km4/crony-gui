use std::borrow::Cow;
use std::fmt::Debug;

use rhai::{Engine, Module, Scope};

use crate::elex::FunctionPtr;

#[derive(Debug)]
pub struct ScriptHost {
    pub(crate) cmd: String,
    pub(crate) history: String,
    is_active: bool,
    engine: Engine,
    scope: Scope<'static>,
}

impl Default for ScriptHost {
    fn default() -> Self {
        let mut engine = Engine::new();
        engine.register_static_module("game", ScriptHost::create_game_module().into());
        Self {
            cmd: String::new(),
            history: String::new(),
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

        let out = self.handle_command();
        self.history.push_str(&out);
        self.cmd.clear();
    }

    fn handle_command(&mut self) -> Cow<'static, str> {
        let res = self.engine.run_with_scope(&mut self.scope, &self.cmd);
        match res {
            Ok(()) => Cow::Borrowed(""),
            Err(err) => Cow::Owned(err.to_string() + "\n"),
        }
    }

    fn create_game_module() -> Module {
        let mut module = Module::new();

        for (name, ptr) in crate::elex::get_all_functions() {
            if let Some(custom_handler) = crate::handlers::get_custom_handler(name.as_ref()) {
                custom_handler(name.as_ref(), &mut module, ptr.clone());
            } else {
                module.set_native_fn(name.as_ref(), move || Ok(ptr.invoke_default(())));
            }
        }
        module
    }

    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

pub type CustomHandler = fn(&str, &mut Module, FunctionPtr) -> u64;

#[macro_export]
macro_rules! custom_handler {
    ($ty:ty) => {
        |name, module, ptr| module.set_native_fn(name, move |val: $ty| Ok(ptr.invoke_default(val)))
    };
}
