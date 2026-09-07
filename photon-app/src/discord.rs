use std::{
    sync::{
        OnceLock,
        mpsc::{Receiver, RecvTimeoutError, Sender, channel},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, Assets, Timestamps},
};

/// Discord application client ID used for rich presence.
///
/// The application must have an art asset named `photon` (the Photon logo)
/// uploaded under Rich Presence → Art Assets, otherwise presence shows
/// text only.
pub const DISCORD_CLIENT_ID: &str = "1546156525029036155";

/// Large image key shown in rich presence. Must match an art asset uploaded
/// to the Discord application above.
pub const DISCORD_LARGE_IMAGE: &str = "photon";

/// What the presence should currently display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceUpdate {
    /// e.g. `Editing main.rs`
    pub details: String,
    /// e.g. workspace folder name
    pub state: String,
    /// Art-asset key (from the Photon Discord app) for the active file type,
    /// shown as the small image badge on the large Photon logo. `None` when no
    /// file (or an unknown type) is open.
    pub small_image_key: Option<String>,
    /// Whether presence is enabled in settings.
    pub enabled: bool,
}

/// Map a file path to a Discord art-asset key for its type. Keys match the
/// icons uploaded to the Photon application (short names like `rs`, `py`).
/// Returns `None` for unknown extensions so the RPC falls back to no badge.
pub fn asset_key_for_path(path: &std::path::Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let key = match ext.as_str() {
        "rs" => "rs",
        "py" | "pyi" | "pyw" => "py",
        "js" | "mjs" | "cjs" | "jsx" => "js",
        "ts" | "mts" | "cts" | "tsx" => "ts",
        "json" | "jsonc" | "json5" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yml",
        "css" => "css",
        "scss" => "sass",
        "sass" => "sass",
        "less" => "less",
        "html" | "htm" | "xhtml" => "html",
        "vue" => "vue",
        "md" | "markdown" | "mdx" => "md",
        "sh" | "bash" | "zsh" | "fish" | "ksh" => "sh",
        "svg" => "svg",
        "xml" | "xsd" | "xsl" => "xml",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "cpp",
        "go" => "go",
        "java" => "java",
        "php" => "php",
        "dockerfile" | "dockerignore" => "docker",
        "gitignore" | "gitattributes" | "gitmodules" | "git" => "git",
        "conf" | "config" | "ini" | "properties" | "editorconfig" => "config",
        _ => return None,
    };
    Some(key)
}

fn worker(rx: Receiver<PresenceUpdate>) {
    let session_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut client: Option<DiscordIpcClient> = None;
    let mut last: Option<PresenceUpdate> = None;

    let ensure_connected = |client: &mut Option<DiscordIpcClient>| -> bool {
        if client.is_some() {
            return true;
        }
        match DiscordIpcClient::new(DISCORD_CLIENT_ID) {
            Ok(mut c) => match c.connect() {
                Ok(()) => {
                    *client = Some(c);
                    true
                }
                Err(err) => {
                    tracing::warn!("Discord presence: connect failed: {err}");
                    false
                }
            },
            Err(err) => {
                tracing::warn!("Discord presence: client failed: {err}");
                false
            }
        }
    };

    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(update) => {
                // Temporary diagnostic for presence update issues.
                if std::env::var("PHOTON_PRESENCE_DEBUG").is_ok() {
                    eprintln!(
                        "PRESENCE_RX details={:?} state={:?} enabled={}",
                        update.details, update.state, update.enabled
                    );
                }
                if last.as_ref() == Some(&update) {
                    continue;
                }
                last = Some(update.clone());
                if !update.enabled {
                    if let Some(c) = client.as_mut() {
                        let _ = c.clear_activity();
                    }
                    continue;
                }
                if !ensure_connected(&mut client) {
                    continue;
                }
                let mut assets = Assets::new()
                    .large_image(DISCORD_LARGE_IMAGE)
                    .large_text("Photon");
                if let Some(key) = &update.small_image_key {
                    assets = assets.small_image(key.as_str()).small_text(&update.details);
                }
                let activity = Activity::new()
                    .details(&update.details)
                    .state(&update.state)
                    .assets(assets)
                    .timestamps(Timestamps::new().start(session_start));
                if let Some(c) = client.as_mut() {
                    if let Err(err) = c.set_activity(activity) {
                        tracing::warn!("Discord presence: set failed: {err}");
                        if let Some(mut c) = client.take() {
                            let _ = c.close();
                        }
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // Reconnect retry tick when Discord was unreachable.
                if last.as_ref().is_some_and(|u| u.enabled) {
                    ensure_connected(&mut client);
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

static SENDER: OnceLock<Sender<PresenceUpdate>> = OnceLock::new();

/// Global sender for presence updates. Spawns the worker thread on first use.
/// Call from the main thread; reading signals must happen by the caller.
pub fn presence_sender() -> Sender<PresenceUpdate> {
    SENDER
        .get_or_init(|| {
            let (tx, rx) = channel();
            std::thread::Builder::new()
                .name("photon-discord-presence".to_string())
                .spawn(move || worker(rx))
                .expect("spawn discord presence thread");
            tx
        })
        .clone()
}
