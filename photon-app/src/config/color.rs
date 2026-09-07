use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadThemeError {
    #[error("themes folder not found, possibly it could not be created")]
    ThemesFolderNotFound,
    #[error("theme file ({theme_name}.toml) was not found in {themes_folder:?}")]
    FileNotFound {
        themes_folder: PathBuf,
        theme_name: String,
    },
    #[error("recursion limit reached for {variable_name}")]
    RecursionLimitReached { variable_name: String },
    #[error("variable {variable_name} not found")]
    VariableNotFound { variable_name: String },
    #[error("There was an error reading the theme file")]
    Read(std::io::Error),
}

pub struct PhotonColor {}

impl PhotonColor {
    pub const PHOTON_WARN: &'static str = "photon.warn";
    pub const PHOTON_ERROR: &'static str = "photon.error";
    pub const PHOTON_DROPDOWN_SHADOW: &'static str = "photon.dropdown_shadow";
    pub const PHOTON_BORDER: &'static str = "photon.border";
    pub const PHOTON_SCROLL_BAR: &'static str = "photon.scroll_bar";

    pub const PHOTON_BUTTON_PRIMARY_BACKGROUND: &'static str =
        "photon.button.primary.background";
    pub const PHOTON_BUTTON_PRIMARY_FOREGROUND: &'static str =
        "photon.button.primary.foreground";

    pub const PHOTON_TAB_ACTIVE_BACKGROUND: &'static str =
        "photon.tab.active.background";
    pub const PHOTON_TAB_ACTIVE_FOREGROUND: &'static str =
        "photon.tab.active.foreground";
    pub const PHOTON_TAB_ACTIVE_UNDERLINE: &'static str =
        "photon.tab.active.underline";

    pub const PHOTON_TAB_INACTIVE_BACKGROUND: &'static str =
        "photon.tab.inactive.background";
    pub const PHOTON_TAB_INACTIVE_FOREGROUND: &'static str =
        "photon.tab.inactive.foreground";
    pub const PHOTON_TAB_INACTIVE_UNDERLINE: &'static str =
        "photon.tab.inactive.underline";

    pub const PHOTON_TAB_SEPARATOR: &'static str = "photon.tab.separator";

    pub const PHOTON_ICON_ACTIVE: &'static str = "photon.icon.active";
    pub const PHOTON_ICON_INACTIVE: &'static str = "photon.icon.inactive";

    pub const PHOTON_REMOTE_ICON: &'static str = "photon.remote.icon";
    pub const PHOTON_REMOTE_LOCAL: &'static str = "photon.remote.local";
    pub const PHOTON_REMOTE_CONNECTED: &'static str = "photon.remote.connected";
    pub const PHOTON_REMOTE_CONNECTING: &'static str = "photon.remote.connecting";
    pub const PHOTON_REMOTE_DISCONNECTED: &'static str = "photon.remote.disconnected";

    pub const PHOTON_PLUGIN_NAME: &'static str = "photon.plugin.name";
    pub const PHOTON_PLUGIN_DESCRIPTION: &'static str = "photon.plugin.description";
    pub const PHOTON_PLUGIN_AUTHOR: &'static str = "photon.plugin.author";

    pub const EDITOR_BACKGROUND: &'static str = "editor.background";
    pub const EDITOR_FOREGROUND: &'static str = "editor.foreground";
    pub const EDITOR_DIM: &'static str = "editor.dim";
    pub const EDITOR_FOCUS: &'static str = "editor.focus";
    pub const EDITOR_CARET: &'static str = "editor.caret";
    pub const EDITOR_SELECTION: &'static str = "editor.selection";
    pub const EDITOR_DEBUG_BREAK_LINE: &'static str = "editor.debug_break_line";
    pub const EDITOR_CURRENT_LINE: &'static str = "editor.current_line";
    pub const EDITOR_LINK: &'static str = "editor.link";
    pub const EDITOR_VISIBLE_WHITESPACE: &'static str = "editor.visible_whitespace";
    pub const EDITOR_INDENT_GUIDE: &'static str = "editor.indent_guide";
    pub const EDITOR_DRAG_DROP_BACKGROUND: &'static str =
        "editor.drag_drop_background";
    pub const EDITOR_STICKY_HEADER_BACKGROUND: &'static str =
        "editor.sticky_header_background";
    pub const EDITOR_DRAG_DROP_TAB_BACKGROUND: &'static str =
        "editor.drag_drop_tab_background";

    pub const INLAY_HINT_FOREGROUND: &'static str = "inlay_hint.foreground";
    pub const INLAY_HINT_BACKGROUND: &'static str = "inlay_hint.background";

    pub const ERROR_LENS_ERROR_FOREGROUND: &'static str =
        "error_lens.error.foreground";
    pub const ERROR_LENS_ERROR_BACKGROUND: &'static str =
        "error_lens.error.background";
    pub const ERROR_LENS_WARNING_FOREGROUND: &'static str =
        "error_lens.warning.foreground";
    pub const ERROR_LENS_WARNING_BACKGROUND: &'static str =
        "error_lens.warning.background";
    pub const ERROR_LENS_OTHER_FOREGROUND: &'static str =
        "error_lens.other.foreground";
    pub const ERROR_LENS_OTHER_BACKGROUND: &'static str =
        "error_lens.other.background";

    pub const COMPLETION_LENS_FOREGROUND: &'static str =
        "completion_lens.foreground";

    pub const SOURCE_CONTROL_ADDED: &'static str = "source_control.added";
    pub const SOURCE_CONTROL_REMOVED: &'static str = "source_control.removed";
    pub const SOURCE_CONTROL_MODIFIED: &'static str = "source_control.modified";

    pub const TERMINAL_CURSOR: &'static str = "terminal.cursor";
    pub const TERMINAL_BACKGROUND: &'static str = "terminal.background";
    pub const TERMINAL_FOREGROUND: &'static str = "terminal.foreground";
    pub const TERMINAL_RED: &'static str = "terminal.red";
    pub const TERMINAL_BLUE: &'static str = "terminal.blue";
    pub const TERMINAL_GREEN: &'static str = "terminal.green";
    pub const TERMINAL_YELLOW: &'static str = "terminal.yellow";
    pub const TERMINAL_BLACK: &'static str = "terminal.black";
    pub const TERMINAL_WHITE: &'static str = "terminal.white";
    pub const TERMINAL_CYAN: &'static str = "terminal.cyan";
    pub const TERMINAL_MAGENTA: &'static str = "terminal.magenta";

    pub const TERMINAL_BRIGHT_RED: &'static str = "terminal.bright_red";
    pub const TERMINAL_BRIGHT_BLUE: &'static str = "terminal.bright_blue";
    pub const TERMINAL_BRIGHT_GREEN: &'static str = "terminal.bright_green";
    pub const TERMINAL_BRIGHT_YELLOW: &'static str = "terminal.bright_yellow";
    pub const TERMINAL_BRIGHT_BLACK: &'static str = "terminal.bright_black";
    pub const TERMINAL_BRIGHT_WHITE: &'static str = "terminal.bright_white";
    pub const TERMINAL_BRIGHT_CYAN: &'static str = "terminal.bright_cyan";
    pub const TERMINAL_BRIGHT_MAGENTA: &'static str = "terminal.bright_magenta";

    pub const PALETTE_BACKGROUND: &'static str = "palette.background";
    pub const PALETTE_FOREGROUND: &'static str = "palette.foreground";
    pub const PALETTE_CURRENT_BACKGROUND: &'static str =
        "palette.current.background";
    pub const PALETTE_CURRENT_FOREGROUND: &'static str =
        "palette.current.foreground";

    pub const COMPLETION_BACKGROUND: &'static str = "completion.background";
    pub const COMPLETION_CURRENT: &'static str = "completion.current";

    pub const HOVER_BACKGROUND: &'static str = "hover.background";

    pub const ACTIVITY_BACKGROUND: &'static str = "activity.background";
    pub const ACTIVITY_CURRENT: &'static str = "activity.current";

    pub const DEBUG_BREAKPOINT: &'static str = "debug.breakpoint";
    pub const DEBUG_BREAKPOINT_HOVER: &'static str = "debug.breakpoint.hover";

    pub const TOOLTIP_BACKGROUND: &'static str = "tooltip.background";
    pub const TOOLTIP_FOREGROUND: &'static str = "tooltip.foreground";

    pub const PANEL_BACKGROUND: &'static str = "panel.background";
    pub const PANEL_FOREGROUND: &'static str = "panel.foreground";
    pub const PANEL_FOREGROUND_DIM: &'static str = "panel.foreground.dim";
    pub const PANEL_CURRENT_BACKGROUND: &'static str = "panel.current.background";
    pub const PANEL_CURRENT_FOREGROUND: &'static str = "panel.current.foreground";
    pub const PANEL_CURRENT_FOREGROUND_DIM: &'static str =
        "panel.current.foreground.dim";
    pub const PANEL_HOVERED_BACKGROUND: &'static str = "panel.hovered.background";
    pub const PANEL_HOVERED_ACTIVE_BACKGROUND: &'static str =
        "panel.hovered.active.background";
    pub const PANEL_HOVERED_FOREGROUND: &'static str = "panel.hovered.foreground";
    pub const PANEL_HOVERED_FOREGROUND_DIM: &'static str =
        "panel.hovered.foreground.dim";

    pub const STATUS_BACKGROUND: &'static str = "status.background";
    pub const STATUS_FOREGROUND: &'static str = "status.foreground";
    pub const STATUS_MODAL_NORMAL_BACKGROUND: &'static str =
        "status.modal.normal.background";
    pub const STATUS_MODAL_NORMAL_FOREGROUND: &'static str =
        "status.modal.normal.foreground";
    pub const STATUS_MODAL_INSERT_BACKGROUND: &'static str =
        "status.modal.insert.background";
    pub const STATUS_MODAL_INSERT_FOREGROUND: &'static str =
        "status.modal.insert.foreground";
    pub const STATUS_MODAL_VISUAL_BACKGROUND: &'static str =
        "status.modal.visual.background";
    pub const STATUS_MODAL_VISUAL_FOREGROUND: &'static str =
        "status.modal.visual.foreground";
    pub const STATUS_MODAL_TERMINAL_BACKGROUND: &'static str =
        "status.modal.terminal.background";
    pub const STATUS_MODAL_TERMINAL_FOREGROUND: &'static str =
        "status.modal.terminal.foreground";

    pub const MARKDOWN_BLOCKQUOTE: &'static str = "markdown.blockquote";
}
