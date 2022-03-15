use std::any::TypeId;
use std::fmt::Debug;
use std::sync::mpsc;

use egui::{Align2, Color32, Context, FontId, Key, Rounding, Sense, TextEdit, Vec2, Window};
use flexi_logger::{FileSpec, LogSpecification, Logger, WriteMode};
use heck::ToSnakeCase;
use rhai::plugin::CallableFunction;
use rhai::{Dynamic, Engine, FnAccess, FnNamespace, Module, RegisterNativeFunction, Scope};

use crate::{elex, handlers};

#[derive(Debug)]
pub struct ScriptHost {
    cmd: String,
    history: String,
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
            log::info!("{}", str);
            self.history.push_str(&str);
            self.history.push('\n');
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
        module
    }
}

impl egui_hook::App for ScriptHost {
    fn render(&mut self, ctx: &Context) {
        const DEFAULT_SIZE: Vec2 = Vec2::new(600., 320.);

        let was_active = self.is_active();
        if ctx.input().key_pressed(Key::Home) {
            self.toggle();
        }
        if !self.is_active() {
            return;
        }

        Window::new("CRONY GUI")
            .default_size(DEFAULT_SIZE)
            .show(ctx, |ui| {
                let (resp, painter) =
                    ui.allocate_painter(DEFAULT_SIZE, Sense::focusable_noninteractive());

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
        Logger::with(LogSpecification::info())
            .log_to_file(
                FileSpec::default()
                    .directory("plugins/logs")
                    .basename("crony"),
            )
            .write_mode(WriteMode::BufferAndFlush)
            .start()
            .ok();

        if elex::version_check() {
            log::info!("C.R.O.N.Y successfully initialized!");
            true
        } else {
            log::error!("Unsupported game version, exiting!");
            false
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
