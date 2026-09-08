//! Built-in zero-config language servers.
//!
//! Photon can start language servers it finds in `PATH` all by itself, with
//! no plugin to install and nothing to configure: open a file, and if a
//! known server binary for its language exists, it just works. Servers
//! started this way behave like any plugin-spawned server (hover,
//! completion, diagnostics, document highlights, ...).
//!
//! A plugin (volt) always wins: when one claims a language, no built-in
//! server is started for it.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    thread,
};

use lsp_types::{DocumentFilter, DocumentSelector, Url};
use photon_rpc::plugin::VoltID;

use super::{PluginCatalogRpcHandler, lsp::LspClient};

/// Reserved plugin-configuration entry the app uses to tell the proxy
/// whether built-in servers are enabled.
pub const BUILTIN_LSP_CONFIG_NAME: &str = "photon-builtin-lsp";
pub const BUILTIN_LSP_ENABLE_KEY: &str = "enable";

/// Identity built-in servers register under, so the catalog can tell them
/// apart from plugin-spawned servers.
pub const BUILTIN_VOLT_AUTHOR: &str = "photon";

pub fn builtin_volt_name(language_id: &str) -> String {
    format!("builtin-lsp-{language_id}")
}

pub struct BuiltinServer {
    pub program: &'static str,
    pub args: &'static [&'static str],
}

/// language id -> candidate servers, first binary found in `PATH` wins.
pub static BUILTIN_SERVERS: &[(&str, &[BuiltinServer])] = &[
    (
        "ada",
        &[BuiltinServer {
            program: "ada_language_server",
            args: &[],
        }],
    ),
    (
        "astro",
        &[BuiltinServer {
            program: "astro-ls",
            args: &["--stdio"],
        }],
    ),
    (
        "c",
        &[BuiltinServer {
            program: "clangd",
            args: &[],
        }],
    ),
    (
        "clojure",
        &[BuiltinServer {
            program: "clojure-lsp",
            args: &[],
        }],
    ),
    (
        "cmake",
        &[BuiltinServer {
            program: "cmake-language-server",
            args: &[],
        }],
    ),
    (
        "cpp",
        &[BuiltinServer {
            program: "clangd",
            args: &[],
        }],
    ),
    (
        "crystal",
        &[BuiltinServer {
            program: "crystalline",
            args: &[],
        }],
    ),
    (
        "csharp",
        &[
            BuiltinServer {
                program: "csharp-ls",
                args: &[],
            },
            BuiltinServer {
                program: "omnisharp",
                args: &["-lsp"],
            },
            BuiltinServer {
                program: "OmniSharp",
                args: &["-lsp"],
            },
        ],
    ),
    (
        "css",
        &[BuiltinServer {
            program: "vscode-css-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "dart",
        &[BuiltinServer {
            program: "dart",
            args: &["language-server", "--protocol=lsp"],
        }],
    ),
    (
        "dlang",
        &[BuiltinServer {
            program: "serve-d",
            args: &[],
        }],
    ),
    (
        "dockerfile",
        &[BuiltinServer {
            program: "docker-langserver",
            args: &["--stdio"],
        }],
    ),
    (
        "elixir",
        &[
            BuiltinServer {
                program: "elixir-ls",
                args: &[],
            },
            BuiltinServer {
                program: "language_server.sh",
                args: &[],
            },
            BuiltinServer {
                program: "expert",
                args: &[],
            },
            BuiltinServer {
                program: "nextls",
                args: &["--stdio"],
            },
        ],
    ),
    (
        "elm",
        &[BuiltinServer {
            program: "elm-language-server",
            args: &[],
        }],
    ),
    (
        "erlang",
        &[BuiltinServer {
            program: "erlang_ls",
            args: &[],
        }],
    ),
    (
        "fortran",
        &[BuiltinServer {
            program: "fortls",
            args: &[],
        }],
    ),
    (
        "fsharp",
        &[BuiltinServer {
            program: "fsautocomplete",
            args: &[],
        }],
    ),
    (
        "glsl",
        &[BuiltinServer {
            program: "glsl_analyzer",
            args: &[],
        }],
    ),
    (
        "go",
        &[BuiltinServer {
            program: "gopls",
            args: &[],
        }],
    ),
    (
        "graphql",
        &[BuiltinServer {
            program: "graphql-lsp",
            args: &["server", "-m", "stream"],
        }],
    ),
    (
        "handlebars",
        &[BuiltinServer {
            program: "ember-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "haskell",
        &[BuiltinServer {
            program: "haskell-language-server-wrapper",
            args: &["--lsp"],
        }],
    ),
    (
        "html",
        &[BuiltinServer {
            program: "vscode-html-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "java",
        &[BuiltinServer {
            program: "jdtls",
            args: &[],
        }],
    ),
    (
        "javascript",
        &[
            BuiltinServer {
                program: "typescript-language-server",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "deno",
                args: &["lsp"],
            },
        ],
    ),
    (
        "javascriptreact",
        &[
            BuiltinServer {
                program: "typescript-language-server",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "deno",
                args: &["lsp"],
            },
        ],
    ),
    (
        "json",
        &[BuiltinServer {
            program: "vscode-json-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "jsonc",
        &[BuiltinServer {
            program: "vscode-json-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "kotlin",
        &[BuiltinServer {
            program: "kotlin-language-server",
            args: &[],
        }],
    ),
    (
        "kotlinbuildscript",
        &[BuiltinServer {
            program: "kotlin-language-server",
            args: &[],
        }],
    ),
    (
        "lean",
        &[BuiltinServer {
            program: "lean",
            args: &["--server"],
        }],
    ),
    (
        "less",
        &[BuiltinServer {
            program: "vscode-css-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "lua",
        &[BuiltinServer {
            program: "lua-language-server",
            args: &[],
        }],
    ),
    (
        "markdown",
        &[BuiltinServer {
            program: "marksman",
            args: &["server"],
        }],
    ),
    (
        "nim",
        &[BuiltinServer {
            program: "nimlangserver",
            args: &[],
        }],
    ),
    (
        "nix",
        &[
            BuiltinServer {
                program: "nil",
                args: &[],
            },
            BuiltinServer {
                program: "nixd",
                args: &[],
            },
        ],
    ),
    (
        "objective-c",
        &[BuiltinServer {
            program: "clangd",
            args: &[],
        }],
    ),
    (
        "objective-cpp",
        &[BuiltinServer {
            program: "clangd",
            args: &[],
        }],
    ),
    (
        "ocaml",
        &[BuiltinServer {
            program: "ocamllsp",
            args: &[],
        }],
    ),
    (
        "odin",
        &[BuiltinServer {
            program: "ols",
            args: &[],
        }],
    ),
    (
        "perl",
        &[BuiltinServer {
            program: "perlnavigator",
            args: &[],
        }],
    ),
    (
        "php",
        &[
            BuiltinServer {
                program: "intelephense",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "phpactor",
                args: &["language-server"],
            },
        ],
    ),
    (
        "prisma",
        &[BuiltinServer {
            program: "prisma-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "proto",
        &[BuiltinServer {
            program: "protols",
            args: &[],
        }],
    ),
    (
        "purescript",
        &[BuiltinServer {
            program: "purescript-language-server",
            args: &[],
        }],
    ),
    (
        "python",
        &[
            BuiltinServer {
                program: "basedpyright-langserver",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "pyright-langserver",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "pylsp",
                args: &[],
            },
            BuiltinServer {
                program: "ruff",
                args: &["server"],
            },
            BuiltinServer {
                program: "ty",
                args: &["server"],
            },
        ],
    ),
    (
        "qml",
        &[BuiltinServer {
            program: "qmlls",
            args: &[],
        }],
    ),
    (
        "ruby",
        &[
            BuiltinServer {
                program: "ruby-lsp",
                args: &[],
            },
            BuiltinServer {
                program: "solargraph",
                args: &["stdio"],
            },
        ],
    ),
    (
        "rust",
        &[BuiltinServer {
            program: "rust-analyzer",
            args: &[],
        }],
    ),
    (
        "scala",
        &[BuiltinServer {
            program: "metals",
            args: &[],
        }],
    ),
    (
        "scss",
        &[BuiltinServer {
            program: "vscode-css-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "shellscript",
        &[BuiltinServer {
            program: "bash-language-server",
            args: &["start"],
        }],
    ),
    (
        "sql",
        &[BuiltinServer {
            program: "sqls",
            args: &[],
        }],
    ),
    (
        "starlark",
        &[BuiltinServer {
            program: "starpls",
            args: &[],
        }],
    ),
    (
        "svelte",
        &[BuiltinServer {
            program: "svelte-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "swift",
        &[BuiltinServer {
            program: "sourcekit-lsp",
            args: &[],
        }],
    ),
    (
        "tex",
        &[BuiltinServer {
            program: "texlab",
            args: &[],
        }],
    ),
    (
        "toml",
        &[BuiltinServer {
            program: "taplo",
            args: &["lsp", "stdio"],
        }],
    ),
    (
        "typescript",
        &[
            BuiltinServer {
                program: "typescript-language-server",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "deno",
                args: &["lsp"],
            },
        ],
    ),
    (
        "typescriptreact",
        &[
            BuiltinServer {
                program: "typescript-language-server",
                args: &["--stdio"],
            },
            BuiltinServer {
                program: "deno",
                args: &["lsp"],
            },
        ],
    ),
    (
        "typst",
        &[BuiltinServer {
            program: "tinymist",
            args: &[],
        }],
    ),
    (
        "verilog",
        &[BuiltinServer {
            program: "verible-verilog-ls",
            args: &[],
        }],
    ),
    (
        "vhdl",
        &[BuiltinServer {
            program: "vhdl_ls",
            args: &[],
        }],
    ),
    (
        "vue",
        &[BuiltinServer {
            program: "vue-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "xml",
        &[BuiltinServer {
            program: "lemminx",
            args: &[],
        }],
    ),
    (
        "xsl",
        &[BuiltinServer {
            program: "lemminx",
            args: &[],
        }],
    ),
    (
        "yaml",
        &[BuiltinServer {
            program: "yaml-language-server",
            args: &["--stdio"],
        }],
    ),
    (
        "zig",
        &[BuiltinServer {
            program: "zls",
            args: &[],
        }],
    ),
];

/// Extra directories users commonly install language servers into without
/// putting them on `PATH` (bun/cargo/go bins).
fn extra_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) =
        directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())
    {
        for sub in [".bun/bin", ".cargo/bin", "go/bin", ".local/bin"] {
            dirs.push(home.join(sub));
        }
    }
    dirs
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.is_file()
            && path
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// Find `program` in `PATH` (plus a few well-known install dirs),
/// like a shell would when you type its name.
pub fn find_server_program(program: &str) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .collect();
    dirs.extend(extra_search_dirs());

    #[cfg(windows)]
    let names: Vec<String> = ["", ".exe", ".cmd", ".bat"]
        .iter()
        .map(|ext| format!("{program}{ext}"))
        .collect();
    #[cfg(not(windows))]
    let names = [program.to_string()];

    for dir in dirs {
        for name in &names {
            let candidate = dir.join(name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Whether built-in servers are enabled (defaults to true).
pub fn builtin_enabled(
    plugin_configurations: &HashMap<String, HashMap<String, serde_json::Value>>,
) -> bool {
    plugin_configurations
        .get(BUILTIN_LSP_CONFIG_NAME)
        .and_then(|config| config.get(BUILTIN_LSP_ENABLE_KEY))
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

/// Try to start a built-in server for `language_id`.
///
/// Does nothing when the language has no known server or none of its
/// binaries are installed. Safe to call repeatedly; starting is guarded by
/// the caller so each language is only attempted once per session.
pub fn start_builtin_server(
    plugin_rpc: PluginCatalogRpcHandler,
    workspace: Option<PathBuf>,
    language_id: &str,
) {
    let Some(candidates) = BUILTIN_SERVERS.iter().find_map(|(language, servers)| {
        (*language == language_id).then_some(*servers)
    }) else {
        return;
    };

    for server in candidates {
        if find_server_program(server.program).is_none() {
            continue;
        }
        let document_selector: DocumentSelector = vec![DocumentFilter {
            language: Some(language_id.to_string()),
            scheme: None,
            pattern: None,
        }];
        let volt_id = VoltID {
            author: BUILTIN_VOLT_AUTHOR.to_string(),
            name: builtin_volt_name(language_id),
        };
        let volt_display_name =
            format!("Photon Built-in LSP for {language_id} ({})", server.program);
        let server_uri = match Url::parse(&format!("urn:{}", server.program)) {
            Ok(uri) => uri,
            Err(err) => {
                tracing::error!("Built-in LSP: bad server uri: {err:?}");
                return;
            }
        };
        let args: Vec<String> =
            server.args.iter().map(|arg| arg.to_string()).collect();
        tracing::info!(
            "Built-in LSP: starting `{}` for '{language_id}' (no plugin required)",
            server.program,
        );
        thread::spawn(move || {
            if let Err(err) = LspClient::start(
                plugin_rpc,
                document_selector,
                workspace,
                volt_id,
                volt_display_name,
                None,
                None,
                None,
                server_uri,
                args,
                None,
            ) {
                tracing::error!("Built-in LSP: failed to start: {err:?}");
            }
        });
        return;
    }

    let tried: Vec<&str> = candidates.iter().map(|server| server.program).collect();
    tracing::debug!(
        "Built-in LSP: no server for '{language_id}' installed (looked for: {}). Install one or add a plugin.",
        tried.join(", "),
    );
}
