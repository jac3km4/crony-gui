mod elex;
mod handlers;
mod host;

use egui_hook::egui_hook;
use host::ScriptHost;

egui_hook!(ScriptHost);
