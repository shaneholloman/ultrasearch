use gpui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub struct Theme {
    pub mode: ThemeMode,
}

impl Theme {
    pub fn new(mode: ThemeMode) -> Self {
        Self { mode }
    }

    pub fn detect() -> ThemeMode {
        #[cfg(target_os = "windows")]
        {
            use windows::core::w;
            use windows::Win32::System::Registry::{
                RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
            };

            let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            let value = w!("AppsUseLightTheme");

            let mut data: u32 = 0;
            let mut size = std::mem::size_of::<u32>() as u32;

            let result = unsafe {
                RegGetValueW(
                    HKEY_CURRENT_USER,
                    subkey,
                    value,
                    RRF_RT_REG_DWORD,
                    None,
                    Some(&mut data as *mut _ as *mut _),
                    Some(&mut size),
                )
            };

            if result.is_ok() {
                if data == 1 {
                    return ThemeMode::Light;
                } else {
                    return ThemeMode::Dark;
                }
            }
        }
        ThemeMode::Dark
    }

    pub fn colors(&self) -> ThemeColors {
        match self.mode {
            ThemeMode::Dark => ThemeColors {
                bg: hsla(0.0, 0.0, 0.102, 1.0),           // #1a1a1a
                divider: hsla(0.0, 0.0, 0.2, 1.0),        // #333333
                text_primary: hsla(0.0, 0.0, 0.894, 1.0), // #e4e4e4
                text_secondary: hsla(0.0, 0.0, 0.6, 1.0), // #999999
                match_highlight: hsla(0.1, 0.6, 0.5, 0.4),
                selection_bg: hsla(0.6, 0.5, 0.4, 0.3),
                border: hsla(0.0, 0.0, 0.25, 1.0),
                panel_bg: hsla(0.0, 0.0, 0.13, 1.0),
            },
            ThemeMode::Light => ThemeColors {
                bg: hsla(0.0, 0.0, 0.98, 1.0),            // #fafafa
                divider: hsla(0.0, 0.0, 0.9, 1.0),        // #e5e5e5
                text_primary: hsla(0.0, 0.0, 0.1, 1.0),   // #1a1a1a
                text_secondary: hsla(0.0, 0.0, 0.4, 1.0), // #666666
                match_highlight: hsla(0.1, 0.8, 0.6, 0.4),
                selection_bg: hsla(0.6, 0.6, 0.8, 0.2),
                border: hsla(0.0, 0.0, 0.85, 1.0),
                panel_bg: hsla(0.0, 0.0, 0.95, 1.0),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub bg: Hsla,
    pub divider: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub match_highlight: Hsla,
    pub selection_bg: Hsla,
    pub border: Hsla,
    pub panel_bg: Hsla,
}

use crate::globals::GlobalAppState;

pub fn active_colors(cx: &App) -> ThemeColors {
    if let Some(state) = cx.try_global::<GlobalAppState>() {
        state.theme.read(cx).colors()
    } else {
        Theme::new(ThemeMode::Dark).colors()
    }
}
