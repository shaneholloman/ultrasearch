use crate::actions::CloseShortcuts;
use crate::theme;
use gpui::*;

/// Full-screen overlay that provides a rich help + shortcuts experience.
/// Kept as a dedicated component so we can grow docs, support links, and
/// platform caveats without bloating `main.rs`.
pub struct HelpPanel {
    focus_handle: FocusHandle,
}

impl HelpPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn key_badge(label: &'static str, cx: &mut Context<Self>) -> Div {
        let colors = theme::active_colors(cx);
        div()
            .px_2()
            .py_0p5()
            .rounded_md()
            .bg(colors.panel_bg)
            .border_1()
            .border_color(colors.border)
            .text_color(colors.text_primary)
            .text_size(px(11.))
            .child(label)
    }

    fn section(
        title: &'static str,
        items: &'static [(&'static str, &'static str)],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = theme::active_colors(cx);
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.text_primary)
                    .child(title),
            )
            .children(items.iter().map(|(k, v)| {
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .gap_2()
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child(div().child(*v))
                    .child(Self::key_badge(k, cx))
            }))
    }
}

impl Render for HelpPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);

        let callout = |title: &'static str, body: &'static str| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .p_3()
                .rounded_lg()
                .bg(colors.panel_bg)
                .border_1()
                .border_color(colors.border)
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors.text_primary)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_secondary)
                        .child(body),
                )
        };

        div()
            .absolute()
            .top_0()
            .left_0()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(hsla(0.0, 0.0, 0.0, 0.45))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down_out(cx.listener(|_, _, window, cx| {
                window.dispatch_action(Box::new(CloseShortcuts), cx);
            }))
            .on_action(cx.listener(|_, _: &CloseShortcuts, window, cx| {
                window.dispatch_action(Box::new(CloseShortcuts), cx);
            }))
            .child(
                div()
                    .w(px(840.))
                    .max_h(px(620.))
                    .bg(colors.panel_bg)
                    .rounded_xl()
                    .shadow_2xl()
                    .border_1()
                    .border_color(colors.border)
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .scroll_y()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(18.))
                                    .font_weight(FontWeight::BOLD)
                                    .child("Help & Shortcuts"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.text_secondary)
                                    .child("Esc or click outside to close"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_6()
                            .wrap()
                            .child(Self::section(
                                "Navigation",
                                &[
                                    ("Ctrl/Cmd+K", "Focus search"),
                                    ("Up / Down", "Move selection"),
                                    ("Enter", "Open selected"),
                                    ("Ctrl+1/2/3", "Switch modes"),
                                ],
                                cx,
                            ))
                            .child(Self::section(
                                "Actions",
                                &[
                                    ("Ctrl/Cmd+C", "Copy path"),
                                    ("Ctrl+Shift+C", "Copy file"),
                                    ("Ctrl+Shift+O", "Open folder"),
                                    ("Alt+Enter", "Properties"),
                                ],
                                cx,
                            ))
                            .child(Self::section(
                                "System",
                                &[
                                    ("F1 / Ctrl+/", "Toggle help"),
                                    ("Alt+Space", "Quick Search palette"),
                                    ("Ctrl/Cmd+Q", "Quit"),
                                ],
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .wrap()
                            .child(callout(
                                "Quick Search (Alt+Space)",
                                "Floating palette with history, fuzzy highlights, and keyboard-only navigation. If PowerToys Run owns Alt+Space, rebind there or pick another hotkey.",
                            ))
                            .child(callout(
                                "Tray & Updates",
                                "Tray tooltip shows: Idle, Indexing, Update available, Offline. Updates flow: Check -> Download -> Restart to Update. Opt-in is required before checking.",
                            ))
                            .child(callout(
                                "Docs & Setup",
                                "See docs/FEATURES.md for UI highlights and docs/GRAALVM_SETUP.md for Extractous prerequisites (GraalVM 23.x + checksum).",
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .wrap()
                            .child(callout(
                                "Privacy",
                                "Index data and telemetry stay local unless you explicitly opt in. You can revisit onboarding via Settings > Onboarding.",
                            ))
                            .child(callout(
                                "Support shortcuts",
                                "Need a refresher? Press F1 or Ctrl+/ from anywhere, or use the header Help chip.",
                            )),
                    ),
            )
    }
}
