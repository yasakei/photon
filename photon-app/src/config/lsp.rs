use serde::{Deserialize, Serialize};
use structdesc::FieldNames;

#[derive(FieldNames, Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct LspConfig {
    #[field_names(
        desc = "Automatically detect and start language servers installed in PATH (rust-analyzer, clangd, gopls, typescript-language-server, lua-language-server, ...), with no plugin or extra setup. A plugin always takes precedence when one claims the language. (Photon)"
    )]
    #[serde(default = "default_builtin_lsp")]
    pub builtin: bool,
}

fn default_builtin_lsp() -> bool {
    true
}
