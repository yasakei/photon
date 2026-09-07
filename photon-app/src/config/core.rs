use serde::{Deserialize, Serialize};
use structdesc::FieldNames;

#[derive(FieldNames, Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CoreConfig {
    #[field_names(desc = "Enable modal editing (Vim like)")]
    pub modal: bool,
    #[field_names(desc = "Set the color theme of Photon")]
    pub color_theme: String,
    #[field_names(desc = "Set the icon theme of Photon")]
    pub icon_theme: String,
    #[field_names(
        desc = "Enable customised titlebar and disable OS native one (Linux, BSD, Windows)"
    )]
    pub custom_titlebar: bool,
    #[field_names(
        desc = "Only allow double-click to open files in the file explorer"
    )]
    pub file_explorer_double_click: bool,
    #[field_names(
        desc = "Enable auto-reload for the plugin when its configuration changes."
    )]
    pub auto_reload_plugin: bool,
    #[field_names(
        desc = "Show the current file and workspace in Discord rich presence (requires the Discord desktop app to be running)"
    )]
    #[serde(default = "default_enable_discord_presence")]
    pub enable_discord_presence: bool,
    #[field_names(
        desc = "Make window surfaces translucent so the compositor can blur the wallpaper behind them (frosted glass look; needs a compositor blur rule, e.g. Hyprland). Takes effect on restart."
    )]
    #[serde(default = "default_window_transparent")]
    pub window_transparent: bool,
}

fn default_enable_discord_presence() -> bool {
    true
}

fn default_window_transparent() -> bool {
    true
}
