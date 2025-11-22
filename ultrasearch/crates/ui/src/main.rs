//! UltraSearch - World-class desktop search application
//!
//! A high-performance Windows desktop search engine combining instant filename
//! search with deep content indexing, wrapped in a beautiful native UI.

use gpui::prelude::*;
use gpui::{actions, KeyBinding, *};
use ui::model::state::{BackendMode, SearchAppModel};
use ui::views::preview_view::PreviewView;
use ui::views::results_table::ResultsView;
use ui::views::search_view::SearchView;

actions!(
    ultrasearch,
    [
        FocusSearch,
        ClearSearch,
        SubmitSearch,
        SelectNext,
        SelectPrev,
        OpenSelected,
        ModeMetadata,
        ModeMixed,
        ModeContent,
        CopySelectedPath,
        QuitApp
    ]
);

fn app_bg() -> Hsla {
    hsla(0.0, 0.0, 0.102, 1.0)
}
fn divider_color() -> Hsla {
    hsla(0.0, 0.0, 0.2, 1.0)
}
fn text_primary() -> Hsla {
    hsla(0.0, 0.0, 0.894, 1.0)
}

/// Main application window containing all UI components
struct UltraSearchWindow {
    model: Entity<SearchAppModel>,
    search_view: Entity<SearchView>,
    results_view: Entity<ResultsView>,
    preview_view: Entity<PreviewView>,
    focus_handle: FocusHandle,
}

impl UltraSearchWindow {
    fn new(cx: &mut Context<Self>) -> Self {
        let model = cx.new(SearchAppModel::new);

        let search_view = cx.new(|cx| SearchView::new(model.clone(), cx));
        let results_view = cx.new(|cx| ResultsView::new(model.clone(), cx));
        let preview_view = cx.new(|cx| PreviewView::new(model.clone(), cx));

        let focus_handle = cx.focus_handle();

        Self {
            model,
            search_view,
            results_view,
            preview_view,
            focus_handle,
        }
    }

    fn on_focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.search_view.update(cx, |view, _cx| {
            window.focus(&view.focus_handle());
        });
    }

    fn on_clear_search(&mut self, _: &ClearSearch, _window: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| {
            model.set_query(String::new(), cx);
        });
        self.search_view.update(cx, |view, cx| {
            view.clear_search(cx);
        });
    }

    fn on_select_next(&mut self, _: &SelectNext, _window: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| model.select_next(cx));
    }

    fn on_select_prev(&mut self, _: &SelectPrev, _window: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| model.select_previous(cx));
    }

    fn on_submit_search(&mut self, _: &SubmitSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.search_view.update(cx, |view, _cx| {
            window.focus(&view.focus_handle());
        });
        self.open_selected(window, cx);
    }

    fn on_open_selected(&mut self, _: &OpenSelected, window: &mut Window, cx: &mut Context<Self>) {
        self.open_selected(window, cx);
    }

    fn on_mode_metadata(&mut self, _: &ModeMetadata, _window: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| {
            model.set_backend_mode(BackendMode::MetadataOnly, cx)
        });
    }

    fn on_mode_mixed(&mut self, _: &ModeMixed, _window: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| {
            model.set_backend_mode(BackendMode::Mixed, cx)
        });
    }

    fn on_mode_content(&mut self, _: &ModeContent, _window: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |model, cx| {
            model.set_backend_mode(BackendMode::ContentOnly, cx)
        });
    }

    fn on_copy_selected_path(
        &mut self,
        _: &CopySelectedPath,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(path) = self
            .model
            .read(cx)
            .selected_row()
            .and_then(|hit| hit.path.clone())
        {
            cx.write_to_clipboard(ClipboardItem::new_string(path));
        }
    }

    fn on_quit(&mut self, _: &QuitApp, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn open_selected(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self
            .model
            .read(cx)
            .selected_row()
            .and_then(|hit| hit.path.clone())
        {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer").arg(&path).spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&path).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
            }
        }
    }
}

impl Render for UltraSearchWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_focus_search))
            .on_action(cx.listener(Self::on_clear_search))
            .on_action(cx.listener(Self::on_select_next))
            .on_action(cx.listener(Self::on_select_prev))
            .on_action(cx.listener(Self::on_submit_search))
            .on_action(cx.listener(Self::on_open_selected))
            .on_action(cx.listener(Self::on_mode_metadata))
            .on_action(cx.listener(Self::on_mode_mixed))
            .on_action(cx.listener(Self::on_mode_content))
            .on_action(cx.listener(Self::on_copy_selected_path))
            .on_action(cx.listener(Self::on_quit))
            .size_full()
            .flex()
            .flex_col()
            .bg(app_bg())
            .text_color(text_primary())
            .child(
                // Search header - fixed at top
                div().flex_shrink_0().child(self.search_view.clone()),
            )
            .child(
                // Main content area - flexible height
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .child(
                        // Results table - 60% width
                        div()
                            .flex_basis(relative(0.6))
                            .flex_grow()
                            .overflow_hidden()
                            .border_r_1()
                            .border_color(divider_color())
                            .child(self.results_view.clone()),
                    )
                    .child(
                        // Preview pane - 40% width
                        div()
                            .flex_basis(relative(0.4))
                            .flex_shrink_0()
                            .overflow_hidden()
                            .child(self.preview_view.clone()),
                    ),
            )
    }
}

fn main() {
    // Load configuration
    if let Err(e) = core_types::config::load_or_create_config(None) {
        eprintln!("Failed to load configuration: {}", e);
        eprintln!("Continuing with default configuration...");
    }

    // Initialize GPUI application
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-k", FocusSearch, None),
            KeyBinding::new("ctrl-k", FocusSearch, None),
            KeyBinding::new("escape", ClearSearch, None),
            KeyBinding::new("enter", SubmitSearch, None),
            KeyBinding::new("down", SelectNext, None),
            KeyBinding::new("up", SelectPrev, None),
            KeyBinding::new("cmd-1", ModeMetadata, None),
            KeyBinding::new("ctrl-1", ModeMetadata, None),
            KeyBinding::new("cmd-2", ModeMixed, None),
            KeyBinding::new("ctrl-2", ModeMixed, None),
            KeyBinding::new("cmd-3", ModeContent, None),
            KeyBinding::new("ctrl-3", ModeContent, None),
            KeyBinding::new("cmd-o", OpenSelected, None),
            KeyBinding::new("ctrl-o", OpenSelected, None),
            KeyBinding::new("cmd-c", CopySelectedPath, None),
            KeyBinding::new("ctrl-c", CopySelectedPath, None),
            KeyBinding::new("cmd-q", QuitApp, None),
            KeyBinding::new("ctrl-q", QuitApp, None),
        ]);

        // Open the main window
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(100.0),
                        y: px(100.0),
                    },
                    size: Size {
                        width: px(1400.0),
                        height: px(900.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("UltraSearch")),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                window_min_size: Some(Size {
                    width: px(960.0),
                    height: px(600.0),
                }),
                window_background: WindowBackgroundAppearance::Opaque,
                app_id: Some("com.ultrasearch.desktop".to_string()),
                ..WindowOptions::default()
            },
            |_, cx| cx.new(UltraSearchWindow::new),
        )
        .expect("Failed to open window");

        // Print startup message
        eprintln!("✅ UltraSearch started successfully!");
        eprintln!("🌀 Keyboard shortcuts:");
        eprintln!("   Ctrl/Cmd+K    - Focus search");
        eprintln!("   Escape        - Clear search");
        eprintln!("   ↑/↓           - Navigate results");
        eprintln!("   Ctrl+1/2/3    - Switch search modes");
        eprintln!("   Ctrl+Q        - Quit");

        cx.activate(true);
    });
}
