use egui::{Align2, Color32, Context, FontId, Key, Rounding, Sense, TextEdit, Vec2, Window};
use egui_hook::egui_hook;
use host::ScriptHost;

mod elex;
mod handlers;
mod host;

egui_hook!(ScriptHost, ui);

fn ui(ctx: &Context, app: &mut ScriptHost) {
    const DEFAULT_SIZE: Vec2 = Vec2::new(600., 320.);

    if ctx.input().key_pressed(Key::Home) {
        app.toggle();
    }
    if !app.is_active() {
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
                &app.history,
                FontId::default(),
                Color32::WHITE,
            );

            let input = TextEdit::singleline(&mut app.cmd)
                .code_editor()
                .desired_width(600.)
                .show(ui);

            if ui.input().key_pressed(Key::Enter) {
                input.response.request_focus();
                app.process_command();
            };
        });
}
