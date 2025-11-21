use gpui::prelude::*;
use gpui::*;
use crate::model::state::{SearchAppModel, BackendMode};

pub struct SearchView {
    model: Model<SearchAppModel>,
    focus_handle: FocusHandle,
}

impl SearchView {
    pub fn new(model: Model<SearchAppModel>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        cx.on_focus(&focus_handle, |_, _cx| { /* focus gained */ }).detach();
        
        cx.observe(&model, |_, _, cx| cx.notify()).detach();

        Self {
            model,
            focus_handle,
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let mut query = self.model.read(cx).query.clone();
        let mut changed = false;

        // Handle basic text input
        // Note: This is very primitive. Real implementation should use a proper IME-aware input handler.
        if let Some(char) = event.keystroke.key.chars().next() {
            if !event.keystroke.modifiers.ctrl && !event.keystroke.modifiers.alt && !event.keystroke.modifiers.cmd {
                 if char.is_alphanumeric() || char.is_ascii_punctuation() || char == ' ' {
                     // Check if it's a single char string (gpui sends "a", "b", "space", "backspace")
                     if event.keystroke.key.len() == 1 {
                         query.push(char);
                         changed = true;
                     } else if event.keystroke.key == "space" {
                         query.push(' ');
                         changed = true;
                     }
                 }
            }
        }

        if event.keystroke.key == "backspace" {
            query.pop();
            changed = true;
        }

        if changed {
            self.model.update(cx, |model, cx| {
                model.set_query(query, cx);
            });
        }
    }

    fn set_mode(&mut self, mode: BackendMode, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| {
            model.set_backend_mode(mode, cx);
        });
    }
}

impl Render for SearchView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let query = &model.query;
        let status = &model.status;
        
        let mode_btn = |label: &str, mode: BackendMode, current: BackendMode, cx: &mut Context<Self>| {
            let active = mode == current;
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(if active { rgb(0x4a4a4a) } else { rgb(0x2d2d2d) })
                .text_color(if active { rgb(0xffffff) } else { rgb(0xaaaaaa) })
                .cursor_pointer()
                .child(label)
                .on_click(cx.listener(move |this: &mut Self, _, cx| this.set_mode(mode, cx)))
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .bg(rgb(0x252526))
            .border_b_1()
            .border_color(rgb(0x333333))
            .p_2()
            .child(
                // Input Row
                div()
                    .flex()
                    .items_center()
                    .mb_2()
                    .child(
                        div()
                            .track_focus(&self.focus_handle)
                            .on_key_down(cx.listener(Self::handle_key_down))
                            .flex_1()
                            .bg(rgb(0x3c3c3c))
                            .rounded_md()
                            .p_1()
                            .text_color(rgb(0xffffff))
                            .child(if query.is_empty() { "Type to search..." } else { query.as_str() })
                    )
                    .child(
                        div().ml_2().flex().gap_1()
                            .child(mode_btn("Name", BackendMode::MetadataOnly, status.backend_mode, cx))
                            .child(mode_btn("Mixed", BackendMode::Mixed, status.backend_mode, cx))
                            .child(mode_btn("Content", BackendMode::ContentOnly, status.backend_mode, cx))
                    )
            )
            .child(
                // Status Row
                div()
                    .flex()
                    .text_size(px(12.))
                    .text_color(rgb(0xaaaaaa))
                    .child(format!("{} results", status.total))
                    .child(div().mx_2().child("|"))
                    .child(format!("{} shown", status.shown))
                    .child(div().mx_2().child("|"))
                    .child(if let Some(lat) = status.last_latency_ms {
                        format!("{} ms", lat)
                    } else {
                        "- ms".to_string()
                    })
                    .child(div().mx_2().child("|"))
                    .child(
                        div()
                            .text_color(if status.connected { rgb(0x4caf50) } else { rgb(0xf44336) })
                            .child(if status.connected { "Connected" } else { "Disconnected" })
                    )
            )
    }
}
