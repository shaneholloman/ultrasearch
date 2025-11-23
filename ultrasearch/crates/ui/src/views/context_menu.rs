use crate::theme;
use gpui::*;

pub struct ContextMenu {
    pub items: Vec<ContextMenuItem>,
    pub position: Point<Pixels>,
    pub focus_handle: FocusHandle,
}

pub struct ContextMenuItem {
    pub label: SharedString,
    pub icon: Option<&'static str>,
    pub action: Box<dyn Action>,
}

impl ContextMenu {
    pub fn new(
        position: Point<Pixels>,
        items: Vec<ContextMenuItem>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            items,
            position,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for ContextMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);

        div()
            .track_focus(&self.focus_handle)
            .absolute()
            .top(self.position.y)
            .left(self.position.x)
            .w(px(200.))
            .bg(colors.panel_bg)
            .border_1()
            .border_color(colors.border)
            .rounded_lg()
            .shadow_lg()
            .p_1()
            .flex()
            .flex_col()
            .gap_1()
            .on_mouse_down_out(cx.listener(|_, _, window, _| window.remove_window()))
            .children(self.items.iter().map(|item| {
                let action = item.action.boxed_clone();
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1p5()
                    .rounded_md()
                    .hover(|s| s.bg(colors.selection_bg))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |_, _, window, cx| {
                            cx.dispatch_action(action.as_ref());
                            window.remove_window();
                        }),
                    )
                    .children(item.icon.map(|i| div().child(i)))
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(colors.text_primary)
                            .child(item.label.clone()),
                    )
            }))
    }
}
