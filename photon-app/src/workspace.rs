use std::{collections::HashMap, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{debug::PhotonBreakpoint, main_split::SplitInfo, panel::data::PanelInfo};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct SshHost {
    pub user: Option<String>,
    pub host: String,
    pub port: Option<usize>,
}

impl SshHost {
    pub fn from_string(s: &str) -> Self {
        let mut whole_splits = s.split(':');
        let splits = whole_splits
            .next()
            .unwrap()
            .split('@')
            .collect::<Vec<&str>>();
        let mut splits = splits.iter().rev();
        let host = splits.next().unwrap().to_string();
        let user = splits.next().map(|s| s.to_string());
        let port = whole_splits.next().and_then(|s| s.parse::<usize>().ok());
        Self { user, host, port }
    }

    pub fn user_host(&self) -> String {
        if let Some(user) = self.user.as_ref() {
            format!("{user}@{}", self.host)
        } else {
            self.host.clone()
        }
    }
}

impl Display for SshHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user) = self.user.as_ref() {
            write!(f, "{user}@")?;
        }
        write!(f, "{}", self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{port}")?;
        }
        Ok(())
    }
}

#[cfg(windows)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct WslHost {
    pub host: String,
}

#[cfg(windows)]
impl Display for WslHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.host)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhotonWorkspaceType {
    Local,
    RemoteSSH(SshHost),
    #[cfg(windows)]
    RemoteWSL(WslHost),
}

impl PhotonWorkspaceType {
    pub fn is_local(&self) -> bool {
        matches!(self, PhotonWorkspaceType::Local)
    }

    pub fn is_remote(&self) -> bool {
        use PhotonWorkspaceType::*;

        #[cfg(not(windows))]
        return matches!(self, RemoteSSH(_));

        #[cfg(windows)]
        return matches!(self, RemoteSSH(_) | RemoteWSL(_));
    }
}

impl std::fmt::Display for PhotonWorkspaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhotonWorkspaceType::Local => f.write_str("Local"),
            PhotonWorkspaceType::RemoteSSH(remote) => {
                write!(f, "ssh://{remote}")
            }
            #[cfg(windows)]
            PhotonWorkspaceType::RemoteWSL(remote) => {
                write!(f, "{remote} (WSL)")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotonWorkspace {
    pub kind: PhotonWorkspaceType,
    pub path: Option<PathBuf>,
    pub last_open: u64,
}

impl PhotonWorkspace {
    pub fn display(&self) -> Option<String> {
        let path = self.path.as_ref()?;
        let path = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .to_string();
        let remote = match &self.kind {
            PhotonWorkspaceType::Local => String::new(),
            PhotonWorkspaceType::RemoteSSH(remote) => {
                format!(" [SSH: {}]", remote.host)
            }
            #[cfg(windows)]
            PhotonWorkspaceType::RemoteWSL(remote) => {
                format!(" [WSL: {}]", remote.host)
            }
        };
        Some(format!("{path}{remote}"))
    }
}

impl Default for PhotonWorkspace {
    fn default() -> Self {
        Self {
            kind: PhotonWorkspaceType::Local,
            path: None,
            last_open: 0,
        }
    }
}

impl std::fmt::Display for PhotonWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            self.kind,
            self.path.as_ref().and_then(|p| p.to_str()).unwrap_or("")
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub split: SplitInfo,
    pub panel: PanelInfo,
    pub breakpoints: HashMap<PathBuf, Vec<PhotonBreakpoint>>,
    /// Editor-only zoom delta in px relative to the settings font size.
    /// `#[serde(default)]` keeps old session files loadable.
    #[serde(default)]
    pub editor_zoom_delta: i32,
    /// Plain shell terminal tabs to reopen: each tab holds the launch
    /// workdir of every terminal in it. Empty means "keep default".
    #[serde(default)]
    pub terminal_tabs: Vec<TerminalTabState>,
    #[serde(default)]
    pub active_terminal_tab: usize,
}

/// Persisted state of one terminal tab: the launch workdir of each
/// terminal it contains, plus which terminal was active.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TerminalTabState {
    #[serde(default)]
    pub active: usize,
    #[serde(default)]
    pub workdirs: Vec<Option<PathBuf>>,
}
