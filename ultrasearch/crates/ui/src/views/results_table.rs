use gpui::*;
use ipc::SearchHit;
use crate::model::state::SearchAppModel;

pub struct ResultsView {
    model: Model<SearchAppModel>,
    list_state: ListState,
}

impl ResultsView {
    pub fn new(model: Model<SearchAppModel>, cx: &mut ViewContext<Self>) -> Self {
        let list_state = ListState::new(
            0,
            ListAlignment::Top,
            px(100.),
        );

        // Subscribe to model updates to refresh the list
        cx.observe(&model, |this: &mut Self, model, cx| {
            let count = model.read(cx).results.len();
            this.list_state.reset(count);
            cx.notify();
        })
        .detach();

        Self {
            model,
            list_state,
        }
    }

    fn render_row(ix: usize, hit: &SearchHit, _cx: &Window, _ctx: &Context<Self>) -> AnyElement {
        let name = hit.name.as_deref().unwrap_or("<unknown>");
        let path = hit.path.as_deref().unwrap_or("");
        let size = hit.size.unwrap_or(0).to_string(); 

        div()
            .flex()
            .w_full()
            .child(
                div()
                    .flex_grow(4.)
                    .child(name.to_string())
            )
            .child(
                div()
                    .flex_grow(6.)
                    .child(path.to_string())
            )
            .child(
                div()
                    .flex_grow(2.)
                    .child(size)
            )
            .into_any_element()
    }
}

impl Render for ResultsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.clone();
        div()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(
                list(self.list_state.clone(), move |ix, window, cx| {
                    let model = model.read(cx);
                    if let Some(hit) = model.results.get(ix) {
                        Self::render_row(ix, hit, window, cx)
                    } else {
                        div().into_any_element()
                    }
                }).size_full()
            )
    }
}
