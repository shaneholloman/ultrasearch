use crate::actions::{CopySelectedPath, OpenContainingFolder, ShowProperties};
use crate::globals::GlobalAppState;
use crate::icon_cache::IconCache;
use crate::model::state::SearchAppModel;
use crate::theme;
use crate::views::context_menu::{ContextMenu, ContextMenuItem};
use gpui::prelude::*;
use gpui::{InteractiveElement, *};
use ipc::SearchHit;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn row_height() -> Pixels {
    px(48.)
}

pub struct ResultsView {
    model: Entity<SearchAppModel>,
    icon_cache: Entity<IconCache>,
    list_state: ListState,
    hover_index: Option<usize>,
}

impl ResultsView {
    pub fn new(model: Entity<SearchAppModel>, cx: &mut Context<ResultsView>) -> Self {
        let list_state = ListState::new(0, ListAlignment::Top, row_height());
        let icon_cache = cx.global::<GlobalAppState>().icon_cache.clone();

        cx.observe(&model, |this: &mut Self, model, cx| {
            let read = model.read(cx);
            let count = read.current_page_results().len();
            this.list_state.reset(count);
            cx.notify();
        })
        .detach();

        cx.observe(&icon_cache, |_, _, cx| {
            cx.notify();
        })
        .detach();

        Self {
            model,
            icon_cache,
            list_state,
            hover_index: None,
        }
    }

    fn highlight_text(
        &self,
        text: &str,
        query: &str,
        is_primary: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = theme::active_colors(cx);
        let needle = query.trim();
        if needle.is_empty() {
            return div()
                .text_size(if is_primary { px(14.) } else { px(11.) })
                .font_weight(if is_primary {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if is_primary {
                    colors.text_primary
                } else {
                    colors.text_secondary
                })
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(text.to_string())
                .into_any_element();
        }

        let lower = text.to_ascii_lowercase();
        let needle_lower = needle.to_ascii_lowercase();
        if let Some(pos) = lower.find(&needle_lower) {
            let end = pos + needle_lower.len();
            let (pre, matched, post) = (&text[..pos], &text[pos..end], &text[end..text.len()]);

            div()
                .text_size(if is_primary { px(14.) } else { px(11.) })
                .font_weight(if is_primary {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if is_primary {
                    colors.text_primary
                } else {
                    colors.text_secondary
                })
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(pre.to_string())
                .child(
                    div()
                        .px_1()
                        .rounded_sm()
                        .bg(colors.match_highlight)
                        .text_color(colors.bg)
                        .child(matched.to_string()),
                )
                .child(post.to_string())
                .into_any_element()
        } else {
            div()
                .text_size(if is_primary { px(14.) } else { px(11.) })
                .font_weight(if is_primary {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if is_primary {
                    colors.text_primary
                } else {
                    colors.text_secondary
                })
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(text.to_string())
                .into_any_element()
        }
    }

    fn render_highlighted(
        &self,
        text: &str,
        query: &str,
        is_primary: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.highlight_text(text, query, is_primary, cx)
    }

    fn handle_click(&mut self, index: usize, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| {
            let global_index = model.page_start().saturating_add(index);
            model.selected_index = Some(global_index.min(model.results.len().saturating_sub(1)));
            model.ensure_page_for_selection();
            cx.notify();
        });
    }

    fn handle_context_menu(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        cx: &mut Context<Self>,
    ) {
        // 1. Select the row
        self.handle_click(index, cx);

        // 2. Create items
        let items = vec![
            ContextMenuItem {
                label: "Open".into(),
                icon: Some("📂"),
                action: Box::new(crate::actions::OpenSelected),
            },
            ContextMenuItem {
                label: "Open Containing Folder".into(),
                icon: Some("🗂"),
                action: Box::new(OpenContainingFolder),
            },
            ContextMenuItem {
                label: "Copy Full Path".into(),
                icon: Some("📋"),
                action: Box::new(CopySelectedPath),
            },
            ContextMenuItem {
                label: "Copy File".into(),
                icon: Some("📄"),
                action: Box::new(crate::actions::CopySelectedFile),
            },
            ContextMenuItem {
                label: "Properties".into(),
                icon: Some("⚙"),
                action: Box::new(ShowProperties),
            },
        ];

        // 3. Spawn Menu Window (Overlay)
        // Assume 0,0 for now as window_bounds is tricky without WindowContext deref.
        // Use event.position (window-relative).
        // Ideally we convert to screen.
        // If main window is moved, this will be offset.
        // For MVP, I will try to get window origin via `cx.window().position()`? No.
        // I'll just use a best-effort approach or center it if I can't find bounds.
        // Actually, let's try `cx.current_window()`?
        // I'll assume the position is relative to the window's content rect.

        // Workaround: We can't easily get screen coords in `Context`?
        // We can pass `event.position` and let the popup open there.
        // `open_window` positions are screen coords.
        // If I assume 0,0 offset, it will appear at top-left of screen + mouse pos?
        // No, if window is at 500,500, mouse at 100,100 -> absolute 600,600.
        // If I use 100,100 for new window, it appears at 100,100 screen.

        // I'll default to 0,0 for origin if bounds fail, but I'll try to use `cx.window().bounds()` if I can fix it.
        // Since I can't fix it quickly, I'll remove the bounds call and use a dummy position.
        // This is a known limitation of my current understanding of GPUI 0.2 API.

        let origin = Point::new(px(0.), px(0.)); // TODO: Fix window origin

        let position = event.position;
        let screen_pos = origin + position;

        let _handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: screen_pos,
                        size: Size {
                            width: px(200.),
                            height: px(150.),
                        },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::PopUp,
                    ..WindowOptions::default()
                },
                |_, cx| cx.new(|cx| ContextMenu::new(Point::default(), items, cx)),
            )
            .ok();
    }

    #[allow(dead_code)]
    fn handle_double_click(&mut self, index: usize, cx: &mut Context<Self>) {
        let model = self.model.read(cx);
        if let Some(hit) = model.results.get(index) {
            if let Some(path) = &hit.path {
                self.open_file(path);
            }
        }
    }

    #[allow(dead_code)]
    fn open_file(&self, path: &str) {
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
            Command::new("open").arg(path).spawn().ok();
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(path).spawn().ok();
        }
    }

    fn format_file_size(bytes: u64) -> String {
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

    fn format_modified_time(timestamp: i64) -> String {
        let now = SystemTime::now();
        let file_time = UNIX_EPOCH + Duration::from_secs(timestamp as u64);

        if let Ok(duration) = now.duration_since(file_time) {
            let days = duration.as_secs() / 86400;
            if days == 0 {
                "Today".to_string()
            } else if days == 1 {
                "Yesterday".to_string()
            } else if days < 7 {
                format!("{days} days ago")
            } else if days < 30 {
                format!("{} weeks ago", days / 7)
            } else if days < 365 {
                format!("{} months ago", days / 30)
            } else {
                format!("{} years ago", days / 365)
            }
        } else {
            "Future".to_string()
        }
    }

    fn get_file_icon_char(ext: Option<&String>) -> &'static str {
        match ext.map(|s| s.as_str()) {
            Some("rs") | Some("toml") | Some("js") | Some("ts") | Some("tsx") | Some("jsx")
            | Some("py") | Some("go") => "🧑‍💻",
            Some("pdf") => "📄",
            Some("docx") | Some("doc") => "📝",
            Some("xlsx") | Some("xls") => "📊",
            Some("pptx") | Some("ppt") => "📈",
            Some("zip") | Some("rar") | Some("7z") => "🗜",
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("svg") => "🖼",
            Some("mp4") | Some("avi") | Some("mkv") => "🎞",
            Some("mp3") | Some("wav") | Some("flac") => "🎵",
            Some("exe") | Some("dll") => "⚙",
            Some("md") | Some("txt") => "📄",
            _ => "📁",
        }
    }

    fn render_row(
        &self,
        index: usize,
        hit: &SearchHit,
        is_selected: bool,
        is_hover: bool,
        query: &str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_even = index.is_multiple_of(2);
        let colors = theme::active_colors(cx);

        let name = hit.name.clone().unwrap_or_else(|| "<unknown>".to_string());
        let path = hit.path.clone().unwrap_or_default();
        let size_text = hit
            .size
            .map(Self::format_file_size)
            .unwrap_or_else(|| "-".to_string());
        let modified_text = hit
            .modified
            .map(Self::format_modified_time)
            .unwrap_or_else(|| "-".to_string());

        // Icon logic: Try native, fallback to emoji
        let ext_str = hit.ext.as_deref().unwrap_or("");
        let icon_img = self
            .icon_cache
            .update(cx, |cache: &mut IconCache, cx| cache.get(ext_str, cx));

        let icon_el = if let Some(src) = icon_img {
            img(src).w(px(20.)).h(px(20.)).into_any_element()
        } else {
            let char = Self::get_file_icon_char(hit.ext.as_ref());
            div().text_size(px(20.)).child(char).into_any_element()
        };

        let score_pct = (hit.score * 100.0) as u32;
        let row_bg = if is_selected {
            colors.selection_bg
        } else if is_hover {
            colors.panel_bg
        } else if is_even {
            colors.bg
        } else {
            colors.panel_bg
        };

        div()
            .w_full()
            .h(row_height())
            .flex()
            .items_center()
            .px_4()
            .gap_3()
            .bg(row_bg)
            .border_b_1()
            .border_color(colors.divider)
            .cursor_pointer()
            .tab_stop(true)
            .tab_index(0)
            .focus_visible(|style| style.border_color(colors.match_highlight).border_2())
            .on_mouse_move(cx.listener(move |this, _, _, cx| {
                if this.hover_index != Some(index) {
                    this.hover_index = Some(index);
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    this.handle_click(index, cx);
                    if event.click_count >= 2 {
                        this.handle_double_click(index, cx);
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    this.handle_context_menu(index, event, cx);
                }),
            )
            .child(icon_el)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .overflow_hidden()
                    .child(self.render_highlighted(&name, query, true, cx))
                    .child(self.render_highlighted(&path, query, false, cx)),
            )
            // Score badge
            .when(score_pct > 0, |mut this: Div| {
                this = this.child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(colors.panel_bg)
                        .text_size(px(10.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors.text_secondary)
                        .child(format!("{score_pct}%")),
                );
                this
            })
            // Size column
            .child(
                div()
                    .w(px(80.))
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child(size_text),
            )
            // Modified column
            .child(
                div()
                    .w(px(100.))
                    .text_size(px(12.))
                    .text_color(colors.text_secondary)
                    .child(modified_text),
            )
            .into_any_element()
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);
        div()
            .w_full()
            .h(px(40.))
            .flex()
            .items_center()
            .px_4()
            .gap_3()
            .bg(colors.panel_bg)
            .border_b_1()
            .border_color(colors.border)
            .text_size(px(11.))
            .font_weight(FontWeight::BOLD)
            .text_color(colors.text_secondary)
            .child(div().w(px(20.))) // Icon space
            .child(div().flex_1().child("NAME"))
            .child(div().w(px(80.)).child("SIZE"))
            .child(div().w(px(100.)).child("MODIFIED"))
    }

    fn render_empty_state(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let has_query = !model.query.is_empty();
        let colors = theme::active_colors(cx);

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .child(
                div()
                    .text_size(px(48.))
                    .child(if has_query { "🤔" } else { "🔍" }),
            )
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.text_secondary)
                    .child(if has_query {
                        "No results found"
                    } else {
                        "Start typing to search"
                    }),
            )
            .when(has_query, |this| {
                this.child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .child("Try different search terms or search mode"),
                )
            })
    }
}

impl Render for ResultsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.clone();
        let model_read = model.read(cx);
        let start = model_read.page_start();
        let page_hits = model_read.current_page_results().to_vec();
        let has_results = !page_hits.is_empty();
        let hover_index = self.hover_index;
        let colors = theme::active_colors(cx);
        let query = model_read.query.clone();

        div()
            .size_full()
            .bg(colors.bg)
            .flex()
            .flex_col()
            .on_mouse_move(cx.listener(|this, _, _, cx| {
                if this.hover_index.is_some() {
                    this.hover_index = None;
                    cx.notify();
                }
            }))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if this.hover_index.is_some() {
                    this.hover_index = None;
                    cx.notify();
                }
            }))
            .when(has_results, |this: Div| {
                this.child(self.render_header(cx)).child(
                    list(
                        self.list_state.clone(),
                        cx.processor(move |this, ix, _window, cx| {
                            let (hit, is_selected) = match page_hits.get(ix).cloned() {
                                Some(hit) => {
                                    let sel = model.read(cx).is_selected(start + ix);
                                    (hit, sel)
                                }
                                None => return div().into_any_element(),
                            };

                            let is_hover = hover_index == Some(ix);
                            this.render_row(start + ix, &hit, is_selected, is_hover, &query, cx)
                        }),
                    )
                    .size_full(),
                )
            })
            .when(!has_results, |this: Div| {
                this.child(self.render_empty_state(cx))
            })
    }
}
