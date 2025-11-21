use gpui::*;
use ipc::SearchHit;
use crate::model::state::SearchAppModel;

pub struct ResultsView {
    model: Model<SearchAppModel>,
    list_state: ListState,
    selection: Option<usize>,
}

impl ResultsView {
    pub fn new(model: Model<SearchAppModel>, cx: &mut ViewContext<Self>) -> Self {
        let list_state = ListState::new(
            0,
            ListAlignment::Top,
            px(24.), // Item height
            {
                let model = model.clone();
                move |ix, cx| {
                    let model = model.read(cx);
                    if let Some(hit) = model.results.get(ix) {
                        Self::render_row(ix, hit, cx)
                    } else {
                        div().into_any_element()
                    }
                }
            },
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
            selection: None,
        }
    }

    fn render_row(ix: usize, hit: &SearchHit, _cx: &WindowContext) -> AnyElement {
        // Simple row layout: Name | Path | Size
        // We use flex to distribute space.
        
        let name = hit.name.as_deref().unwrap_or("<unknown>");
        let path = hit.path.as_deref().unwrap_or("");
        let size = hit.size.unwrap_or(0).to_string(); // TODO: format bytes

        div()
            .flex()
            .w_full()
            .child(
                div()
                    .w_4_12() // 33%
                    .child(name.to_string())
            )
            .child(
                div()
                    .w_6_12() // 50%
                    .child(path.to_string())
            )
            .child(
                div()
                    .w_2_12() // 16%
                    .child(size)
            )
            .into_any_element()
    }
}

impl Render for ResultsView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(
                list(self.list_state.clone()).size_full()
            )
    }
}
