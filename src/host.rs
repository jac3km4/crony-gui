use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::{fs, io};

use egui::{Align2, Color32, Context, FontId, Key, Rounding, Sense, TextEdit, Vec2, Window};
use flexi_logger::{Cleanup, Criterion, FileSpec, LogSpecification, Logger, Naming, WriteMode};
use heck::ToSnakeCase;
use rhai::plugin::CallableFunction;
use rhai::*;

use crate::{elex, handlers};

#[derive(Debug)]
pub struct ScriptHost {
    cmd: String,
    history: String,
    log_receiver: mpsc::Receiver<String>,
    is_active: bool,
    engine: Engine,
    scope: Scope<'static>,
    mods: Vec<Mod>,
}

impl Default for ScriptHost {
    fn default() -> Self {
        let mut engine = Engine::new();
        let (tx, tr) = mpsc::channel();

        engine.register_static_module("game", ScriptHost::create_game_module().into());
        engine.register_static_module("entity", ScriptHost::create_entity_module().into());
        engine.register_static_module("item", ScriptHost::create_item_module().into());

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
            mods: vec![],
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
            log::info!("{}", str);
            self.history.push_str(&str);
            self.history.push('\n');
        }
    }

    fn process_frame(&mut self) {
        for mod_ in &mut self.mods {
            let _res: Result<(), _> = self
                .engine
                .call_fn(&mut mod_.scope, &mod_.ast, "on_frame", vec![mod_.state.clone()]);
        }
    }

    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
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
        module.set_native_fn("is_none", |entity: elex::Entity| Ok(entity.is_null()));
        module.set_native_fn("resolve", |name: &str| Ok(elex::resolve_entity(name)));
        module
    }

    fn create_item_module() -> Module {
        let mut module = Module::new();
        module.set_native_fn(
            "give",
            |target: elex::Entity, item: elex::Entity, quantity: i64, x: i64, notify: bool| {
                elex::give_item(&target, &item, quantity as u32, x as u32, notify.into());
                Ok(())
            },
        );
        module
    }

    fn verify_version() -> bool {
        const SUPPORTED_VERSION_TS: u32 = 1647620648;
        let found_version = elex::check_version();

        if found_version == SUPPORTED_VERSION_TS {
            log::info!("C.R.O.N.Y successfully initialized!");
            true
        } else {
            log::error!("Unsupported game version ({found_version}), exiting!");
            false
        }
    }

    fn load_mods(&mut self) -> Result<(), io::Error> {
        let dir = std::env::current_exe()?
            .parent()
            .expect("no exe parent")
            .join("plugins")
            .join("crony");
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let main = entry.path().join("main.rhai");
                if main.exists() {
                    match self.init_mod(main.clone()) {
                        Ok(()) => log::info!("Successfully loaded {}", main.display()),
                        Err(err) => log::info!("Failed to initilize {}: {}", main.display(), err),
                    }
                }
            }
        }

        Ok(())
    }

    fn init_mod(&mut self, path: PathBuf) -> Result<(), Box<EvalAltResult>> {
        let ast = self.engine.compile_file(path)?;
        let mut scope = Scope::new();
        let state = self.engine.call_fn(&mut scope, &ast, "initial_state", ())?;
        self.mods.push(Mod::new(ast, scope, state));
        Ok(())
    }
}

impl egui_hook::App for ScriptHost {
    fn render(&mut self, ctx: &Context) {
        self.process_frame();

        let was_active = self.is_active();
        if ctx.input().key_pressed(Key::Home) {
            self.toggle();
        }
        if !self.is_active() {
            return;
        }

        const DEFAULT_SIZE: Vec2 = Vec2::new(600., 320.);

        Window::new("CRONY GUI")
            .default_size(DEFAULT_SIZE)
            .show(ctx, |ui| {
                let (resp, painter) = ui.allocate_painter(DEFAULT_SIZE, Sense::focusable_noninteractive());

                painter.rect_filled(resp.rect, Rounding::same(4.), Color32::BLACK);
                painter.text(
                    resp.rect.left_bottom(),
                    Align2::LEFT_BOTTOM,
                    &self.history,
                    FontId::monospace(14.),
                    Color32::WHITE,
                );

                let input = TextEdit::singleline(&mut self.cmd)
                    .font(FontId::monospace(14.))
                    .desired_width(600.)
                    .show(ui);

                if self.is_active() != was_active {
                    input.response.request_focus();
                }

                if ui.input().key_pressed(Key::Enter) {
                    input.response.request_focus();
                    self.process_command();
                };
                self.process_events();
            });
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn init() -> bool {
        let log_file = FileSpec::default().directory("plugins/logs").basename("crony");
        Logger::with(LogSpecification::info())
            .log_to_file(log_file)
            .write_mode(WriteMode::BufferAndFlush)
            .rotate(
                Criterion::Size(16380),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(4),
            )
            .start()
            .ok();

        Self::verify_version()
    }

    fn setup(&mut self, _ctx: &Context) {
        if let Err(err) = self.load_mods() {
            log::warn!("Failed to load mods: {err}")
        }
    }
}

#[derive(Debug)]
struct Mod {
    ast: AST,
    scope: Scope<'static>,
    state: Rc<RefCell<Dynamic>>,
}

impl Mod {
    fn new(ast: AST, scope: Scope<'static>, state: Dynamic) -> Self {
        Self {
            ast,
            scope,
            state: Rc::new(RefCell::new(state)),
        }
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
