use crate::actions::ClearSearch;
use crate::model::state::SearchAppModel;
use crate::theme;
use crate::views::results_table::ResultsView;
use crate::views::search_view::SearchView;
use gpui::*;

pub struct QuickBarView {
    search_view: Entity<SearchView>,
    results_view: Entity<ResultsView>,
    focus_handle: FocusHandle,
}

impl QuickBarView {
    pub fn new(model: Entity<SearchAppModel>, cx: &mut Context<Self>) -> Self {
        let search_view = cx.new(|cx| SearchView::new(model.clone(), cx));
        let results_view = cx.new(|cx| ResultsView::new(model.clone(), cx));

        // Use the search view's focus handle as the main handle for this view
        let focus_handle = search_view.read(cx).focus_handle();

        Self {
            search_view,
            results_view,
            focus_handle,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for QuickBarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.panel_bg)
            .border_1()
            .border_color(colors.match_highlight)
            .rounded_xl()
            .shadow_2xl()
            .flex()
            .flex_col()
            .overflow_hidden()
            .key_context("QuickBar")
            .on_action(cx.listener(|_, _: &ClearSearch, window, _cx| {
                window.remove_window();
            }))
            .on_mouse_down_out(cx.listener(|_, _, window, _cx| {
                window.remove_window();
            }))
            .child(div().flex_shrink_0().child(self.search_view.clone()))
            .child(div().flex_1().child(self.results_view.clone()))
    }
}
