use gpui::*;
use crate::model::state::SearchAppModel;
use crate::theme;
use crate::actions::CloseStatus;

pub struct StatusView {
    focus_handle: FocusHandle,
    model: Entity<SearchAppModel>,
}

impl StatusView {
    pub fn new(model: Entity<SearchAppModel>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            model,
        }
    }

    fn render_kv_row(&self, key: &str, value: String, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);
        div()
            .flex()
            .justify_between()
            .py_1()
            .border_b_1()
            .border_color(colors.divider)
            .child(div().text_color(colors.text_secondary).child(key.to_string()))
            .child(div().text_color(colors.text_primary).font_weight(FontWeight::MEDIUM).child(value))
    }

    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{bytes} B")
        }
    }
}

impl Render for StatusView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);
        let model = self.model.read(cx);
        let status = &model.status;

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.bg)
            .text_color(colors.text_primary)
            .flex()
            .items_center()
            .justify_center()
            .on_action(cx.listener(|_, _: &CloseStatus, window, _| {
                // Action handled by window to close view, or model update
                // Wait, we need to close it.
                // Window listener will toggle flag.
            }))
            .child(
                div()
                    .w(px(600.))
                    .h(px(500.)) // Fixed height for now
                    .bg(colors.panel_bg)
                    .border_1()
                    .border_color(colors.border)
                    .rounded_xl()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Header
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .p_4()
                            .border_b_1()
                            .border_color(colors.border)
                            .child(div().text_size(px(18.)).font_weight(FontWeight::BOLD).child("Service Health Dashboard"))
                            .child(
                                div()
                                    .child("✕")
                                    .cursor_pointer()
                                    .text_color(colors.text_secondary)
                                    .hover(|s| s.text_color(colors.text_primary))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| {
                                        cx.dispatch_action(&CloseStatus);
                                    }))
                            )
                    )
                    // Body
                    .child(
                        div()
                            .flex_1()
                            .overflow_y_scroll()
                            .p_6()
                            .flex()
                            .flex_col()
                            .gap_6()
                            // Section: General
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().text_size(px(14.)).font_weight(FontWeight::BOLD).text_color(colors.match_highlight).child("General"))
                                    .child(self.render_kv_row("Connection", if status.connected { "Connected".into() } else { "Disconnected".into() }, cx))
                                    .child(self.render_kv_row("Service Host", status.served_by.clone().unwrap_or("-".into()), cx))
                                    .child(self.render_kv_row("Scheduler State", status.indexing_state.clone(), cx))
                            )
                            // Section: Metrics
                            .when(status.metrics.is_some(), |this| {
                                let m = status.metrics.as_ref().unwrap();
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(div().text_size(px(14.)).font_weight(FontWeight::BOLD).text_color(colors.match_highlight).child("Metrics"))
                                        .child(self.render_kv_row("Latency (P50)", format!("{:.2} ms", m.search_latency_ms_p50.unwrap_or(0.0)), cx))
                                        .child(self.render_kv_row("Latency (P95)", format!("{:.2} ms", m.search_latency_ms_p95.unwrap_or(0.0)), cx))
                                        .child(self.render_kv_row("Worker CPU", format!("{:.1}%", m.worker_cpu_pct.unwrap_or(0.0)), cx))
                                        .child(self.render_kv_row("Worker Mem", Self::format_bytes(m.worker_mem_bytes.unwrap_or(0)), cx))
                                        .child(self.render_kv_row("Queue Depth", format!("{}", m.queue_depth.unwrap_or(0)), cx))
                                )
                            })
                            // Section: Volumes
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().text_size(px(14.)).font_weight(FontWeight::BOLD).text_color(colors.match_highlight).child("Volumes"))
                                    .children(status.volumes.iter().map(|v| {
                                        div()
                                            .p_3()
                                            .bg(colors.bg)
                                            .rounded_md()
                                            .border_1()
                                            .border_color(colors.divider)
                                            .child(div().font_weight(FontWeight::BOLD).child(format!("Volume {}", v.volume)))
                                            .child(div().text_size(px(12.)).text_color(colors.text_secondary).child(format!("Indexed: {} files", v.indexed_files)))
                                            .child(div().text_size(px(12.)).text_color(colors.text_secondary).child(format!("Pending: {} files", v.pending_files)))
                                    }))
                            )
                    )
            )
    }
}
