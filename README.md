# soksak-sidecar-browser-chromium

The bundled Chromium engine behind soksak's **Browser ▸ Chromium** tab — an engine
sidecar (shared native module) that the soksak core loads in-process.

## What it is

- Renders real Chromium into browser panes at native speed. Unlike the OS webview,
  the engine is identical on every install.
- **Not an app.** No window of its own, no Dock icon, nothing to launch. It is a
  dylib the core loads when the browser plugin opens it, plus Chromium's standard
  helper subprocesses (renderer/GPU/network) which appear under this name in a
  process list — that is normal Chromium process architecture.
- **Nothing to install by hand.** The `soksak-plugin-browser-chromium` plugin
  declares this sidecar; soksak fetches the pinned release archive automatically
  and verifies its sha256 before installing. A failed verification installs nothing.

## How it connects

- **Protocol** `soksak-spec-sidecar-browser` — requests `create / bounds / load /
  reload / back / forward / hidden / focus / close / popup-mode`, events
  `nav / title / popup-url`. Spoken between the browser plugin and this module;
  the core relays it without interpreting.
- **Hosting ABI** — exported `soksak_sidecar_engine_*` C symbols. At load the core
  compares the module's self-reported interface with the plugin's declaration and
  refuses on mismatch. Once loaded, the module stays resident for the app's lifetime.

## Development

```sh
cargo build --release
./stage.sh ~/.soksak/sidecars/soksak-sidecar-browser-chromium/dist
```

Staging lays out the dylib, the Chromium framework, and the helper `.app` variants
(base / Renderer / GPU / Plugin / Alerts — Chromium launches renderers from the
`… Helper (Renderer).app` sibling bundle).

- Dev staging: `./stage.sh dist` into the identity home's `sidecars/` — the only resolution path (no env binary override).
- Diagnostics: `SOKSAK_SIDECAR_BROWSER_CHROMIUM_NO_TICK=1` (disable the render tick)

## Release

Pushing a `v*` tag builds the dist archive and uploads `…-darwin-arm64.tar.gz`
plus its sha256 as release assets. Plugins pin that URL and hash in their
manifest. Current assets target macOS arm64; other platforms are planned.

## Attribution

Chromium is embedded via CEF (Chromium Embedded Framework) through the
[`cef`](https://crates.io/crates/cef) Rust crate. Chromium and CEF are
BSD-licensed by their respective authors.
