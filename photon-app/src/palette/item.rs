use std::path::PathBuf;

use photon_core::line_ending::LineEnding;
use photon_rpc::dap_types::RunDebugConfig;
use lsp_types::{Range, SymbolKind};

use crate::{
    command::{PhotonCommand, PhotonWorkbenchCommand},
    debug::RunDebugMode,
    editor::location::EditorLocation,
    workspace::{PhotonWorkspace, SshHost},
};

#[derive(Clone, Debug, PartialEq)]
pub struct PaletteItem {
    pub content: PaletteItemContent,
    pub filter_text: String,
    pub score: u32,
    pub indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaletteItemContent {
    PaletteHelp {
        cmd: PhotonWorkbenchCommand,
    },
    File {
        path: PathBuf,
        full_path: PathBuf,
    },
    Line {
        line: usize,
        content: String,
    },
    Command {
        cmd: PhotonCommand,
    },
    Workspace {
        workspace: PhotonWorkspace,
    },
    Reference {
        path: PathBuf,
        location: EditorLocation,
    },
    DocumentSymbol {
        kind: SymbolKind,
        name: String,
        range: Range,
        container_name: Option<String>,
    },
    WorkspaceSymbol {
        kind: SymbolKind,
        name: String,
        container_name: Option<String>,
        location: EditorLocation,
    },
    SshHost {
        host: SshHost,
    },
    #[cfg(windows)]
    WslHost {
        host: crate::workspace::WslHost,
    },
    RunAndDebug {
        mode: RunDebugMode,
        config: RunDebugConfig,
    },
    ColorTheme {
        name: String,
    },
    IconTheme {
        name: String,
    },
    Language {
        name: String,
    },
    LineEnding {
        kind: LineEnding,
    },
    SCMReference {
        name: String,
    },
    TerminalProfile {
        name: String,
        profile: photon_rpc::terminal::TerminalProfile,
    },
}
