use std::fmt::Debug;
use std::sync::mpsc;

use rhai::{Dynamic, Engine, Module, Scope};

use crate::elex::FunctionPtr;

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

        for (name, ptr) in crate::elex::get_all_functions() {
            if let Some(custom_handler) = crate::handlers::get_custom_handler(name.as_ref()) {
                custom_handler(name.as_ref(), &mut module, ptr.clone());
            } else {
                module.set_native_fn(name.as_ref(), move || Ok(ptr.invoke_default(())));
            }
        }
        module
    }
}

pub type CustomHandler = fn(&str, &mut Module, FunctionPtr) -> u64;

#[macro_export]
macro_rules! custom_handler {
    ($ty:ty) => {
        |name, module, ptr| module.set_native_fn(name, move |val: $ty| Ok(ptr.invoke_default(val)))
    };
}
