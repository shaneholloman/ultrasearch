use crate::actions::FinishOnboarding;
use crate::model::state::SearchAppModel;
use crate::theme;
use gpui::*;
use ipc;
use sysinfo::Disks;
use uuid;

pub struct OnboardingView {
    step: usize,
    available_disks: Vec<(String, bool)>, // (Name/Mount, Selected)
    focus_handle: FocusHandle,
    model: Entity<SearchAppModel>,
}

impl OnboardingView {
    pub fn new(model: Entity<SearchAppModel>, cx: &mut Context<Self>) -> Self {
        let mut disks = Vec::new();
        let sys_disks = Disks::new_with_refreshed_list();
        for disk in sys_disks.list() {
            let mount = disk.mount_point().to_string_lossy().to_string();
            // Default to selecting Fixed disks
            let selected = true;
            disks.push((mount, selected));
        }

        Self {
            step: 0,
            available_disks: disks,
            focus_handle: cx.focus_handle(),
            model,
        }
    }

    fn next_step(&mut self, cx: &mut Context<Self>) {
        if self.step < 2 {
            self.step += 1;
            cx.notify();
        } else {
            // Finish
            self.finish(cx);
        }
    }

    fn toggle_disk(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(disk) = self.available_disks.get_mut(index) {
            disk.1 = !disk.1;
            cx.notify();
        }
    }

    fn finish(&mut self, cx: &mut Context<Self>) {
        // 1. Update Config
        if let Ok(mut config) = core_types::config::load_or_create_config(None) {
            config.volumes = self
                .available_disks
                .iter()
                .filter(|(_, selected)| *selected)
                .map(|(name, _)| name.clone())
                .collect();

            // Save to file (default path)
            let target = std::path::PathBuf::from("config/config.toml");
            if let Ok(toml) = toml::to_string_pretty(&config) {
                let _ = std::fs::write(target, toml);
            }
        }

        // 2. Send IPC Reload
        let client = self.model.read(cx).client.clone();
        cx.spawn(|_, _cx: &mut AsyncApp| async move {
            let req = ipc::ReloadConfigRequest {
                id: uuid::Uuid::new_v4(),
            };
            let _ = client.reload_config(req).await;
        })
        .detach();

        cx.dispatch_action(&FinishOnboarding);
    }
}

impl Render for OnboardingView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::active_colors(cx);

        let content = match self.step {
            0 => div()
                .flex()
                .flex_col()
                .gap_4()
                .child(div().text_size(px(24.)).font_weight(FontWeight::BOLD).child("Welcome to UltraSearch"))
                .child(div().child("UltraSearch needs to index your files to provide instant search results."))
                .child(div().child("This process runs in the background and is resource-efficient.")),
            1 => div()
                .flex()
                .flex_col()
                .gap_4()
                .child(div().text_size(px(18.)).font_weight(FontWeight::BOLD).child("Select Drives"))
                .child(div().child("Which drives should UltraSearch index?"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(self.available_disks.iter().enumerate().map(|(i, (name, selected))| {
                            let color = if *selected { colors.text_primary } else { colors.text_secondary };
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().child(if *selected { "[x]" } else { "[ ]" })) // Replace with checkbox widget later
                                .child(div().text_color(color).child(name.clone()))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                    this.toggle_disk(i, cx);
                                }))
                        }))
                ),
            2 => div()
                .flex()
                .flex_col()
                .gap_4()
                .child(div().text_size(px(18.)).font_weight(FontWeight::BOLD).child("All Set!"))
                .child(div().child("UltraSearch will now start indexing. You can start searching immediately, but results will improve as the index builds.")),
            _ => div().child("Done"),
        };

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.bg)
            .text_color(colors.text_primary)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(500.))
                    .bg(colors.panel_bg)
                    .border_1()
                    .border_color(colors.border)
                    .rounded_xl()
                    .shadow_lg()
                    .p_8()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(content)
                    .child(
                        div().flex().justify_end().child(
                            div()
                                .px_4()
                                .py_2()
                                .bg(colors.match_highlight)
                                .rounded_md()
                                .text_color(white())
                                .cursor_pointer()
                                .child(if self.step == 2 { "Finish" } else { "Next" })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.next_step(cx);
                                    }),
                                ),
                        ),
                    ),
            )
    }
}
