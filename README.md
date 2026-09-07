<h1 align="center">
  <img src="extra/images/logo.png?v=2" width=200 height=200/><br>
  Photon
</h1>

<h4 align="center">A personal, opinionated Lapce fork — fast, pretty, and Discord-aware</h4>

> **Note:** Photon is a fork of [Lapce](https://github.com/lapce/lapce) maintained by [@yasakei](https://github.com/yasakei).
> All credit for the original editor goes to the Lapce authors and contributors.
> Photon is distributed under the same [Apache-2.0 License](LICENSE); see [NOTICE](NOTICE) for attribution.

Photon (IPA: /ˈfoʊtɒn/) is written in pure Rust, with a UI in [Floem](https://github.com/lapce/floem). It keeps everything fast about Lapce and adds its own flavor on top.

## What Photon adds over Lapce

* **Built-in Discord rich presence** — shows the file you're editing with a per-language icon badge, live line/column, and workspace name (needs the Discord desktop app running; toggle with `core.enable-discord-presence`)
* **Editor-only zoom** — `Ctrl+=` / `Ctrl+-` scale just the editor text (not the whole UI), `Ctrl+0` resets; zoom persists per workspace
* **Session restore** — reopens your tabs, splits, cursor positions, terminal tabs (with their working directories), and zoom level
* **TokyoNight (LazyVim-style) default theme** with a lualine-style status bar
* **Frosted-glass mode** — translucent window + surfaces for compositor blur (Hyprland, KWin); toggle with `core.window-transparent` (see `extra/linux/hyprland-photon.conf`)
* **Photon branding** — mascot app icon, desktop entry, and bundled JetBrains Mono
* Inherited from Lapce: built-in LSP support, modal (Vim-like) editing, remote development, WASI plugins, built-in terminal

## Installation

No pre-built releases yet — build from source (needs a Rust toolchain):

```sh
cargo build --release
./scripts/install.sh            # install to ~/.local (builds first by default)
./scripts/install.sh --no-build # reuse the existing target/release/photon
./scripts/install.sh --system   # system-wide install (needs sudo)
```

See [docs/building-from-source.md](docs/building-from-source.md) for dependencies.

## Contributing

This is a personal project, but honest feedback and small PRs are welcome. Guidelines live in [`CONTRIBUTING.md`](CONTRIBUTING.md) (inherited from upstream, may not all apply).

Upstream Lapce lives at [lapce/lapce](https://github.com/lapce/lapce) — bugs that exist there too are best reported there.

## License

Photon is released under the Apache License Version 2. You can find a copy of the license text here: [`LICENSE`](LICENSE).
