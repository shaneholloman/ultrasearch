use gpui::*;
use crate::model::state::SearchAppModel;
use std::process::Command;

pub struct PreviewView {
    model: Model<SearchAppModel>,
}

impl PreviewView {
    pub fn new(model: Model<SearchAppModel>, cx: &mut ViewContext<Self>) -> Self {
        cx.observe(&model, |_, _, cx| cx.notify()).detach();
        Self { model }
    }

    fn open_in_explorer(&mut self, path: &str, _cx: &mut ViewContext<Self>) {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg("/select,")
                .arg(path)
                .spawn()
                .ok();
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn()
                .ok();
        }
        #[cfg(target_os = "linux")]
        {
            // Try to select if possible (e.g. dolphin --select), otherwise just open dir
            // xdg-open opens the file/dir. To select, we might need specific FM support.
            // For generic linux, just opening the parent dir is safer.
            if let Some(parent) = std::path::Path::new(path).parent() {
                 Command::new("xdg-open")
                    .arg(parent)
                    .spawn()
                    .ok();
            }
        }
    }
}

impl Render for PreviewView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let selected = model.selected_row();

        let content = if let Some(row) = selected {
            div()
                .flex()
                .flex_col()
                .p_4()
                .gap_4()
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::BOLD)
                        .child(row.name.clone())
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0xaaaaaa))
                        .child(row.path.clone())
                )
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .child(format!("Size: {}", row.size))
                        .child(format!("Modified: {}", row.modified_ts))
                )
                .child(
                    div()
                        .mt_4()
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .bg(rgb(0x333333))
                                .rounded_md()
                                .cursor_pointer()
                                .child("Open in File Manager")
                                .on_click(cx.listener({
                                    let path = row.path.clone();
                                    move |this, _, cx| this.open_in_explorer(&path, cx)
                                }))
                        )
                )
                .children(row.snippet.as_ref().map(|s| {
                    div()
                        .mt_4()
                        .p_2()
                        .bg(rgb(0x111111))
                        .rounded_md()
                        .text_size(px(12.))
                        .child(s.clone())
                }))
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .text_color(rgb(0x666666))
                .child("Select a file to preview")
        };

        div()
            .size_full()
            .bg(rgb(0x252526))
            .border_l_1()
            .border_color(rgb(0x333333))
            .child(content)
    }
}
