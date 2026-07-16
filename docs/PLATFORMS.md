# Cross-platform rendering and presentation

How the Chromium engine sidecar renders and presents pane content on macOS,
Windows, and Linux. This is the authoritative rule for platform work in this
crate; do not derive a different one from older notes.

## The rule

The sidecar owns native presentation. The engine renders offscreen through CEF
(windowless + shared texture); each platform presents the shared texture onto a
module-owned native surface. Presentation stays inside the sidecar and is not
moved to the core — a per-OS presenter behind one interface is the only shape
consistent with keeping the working macOS path unchanged.

The frame path never crosses the host vtable, IPC, or JS (SIDECARS.md §8): the
pixels are blitted from CEF's shared texture straight onto the native surface.

## Interface and per-OS modules

`presenter/mod.rs` is a platform-neutral interface (surface lifecycle, bounds,
hidden, popup, present). `engine.rs` calls only this interface and never a
platform module directly.

- `presenter/macos.rs` — production. IOSurface → Metal blit → CALayer. This is
  the frozen reference path and stays hand-rolled (raw Metal), untouched.
- `presenter/windows.rs`, `presenter/linux.rs` — import CEF's shared texture with
  the `cef` crate's `osr_texture_import` (feature `accelerated_osr`) into a
  `wgpu::Texture`, then render it to a `wgpu::Surface` on the native child window.

CEF's `on_accelerated_paint` hands a different handle per platform (macOS
IOSurface pointer, Windows D3D11 `HANDLE`, Linux DMA-BUF planes). Rather than
hand-roll three native GPU stacks, the new platforms use the crate's unified
importer: `SharedTextureHandle::new(info).import_texture(&device)` returns a
`wgpu::Texture` (Linux DMA-BUF→Vulkan, Windows D3D11→Vulkan interop), with a CPU
fallback. So Windows and Linux share ONE present mechanism (wgpu); only macOS
stays on raw Metal because its working path is the frozen reference. `present`
takes `&AcceleratedPaintInfo` and each presenter consumes it its own way (macOS
reads `shared_texture_io_surface`; wgpu presenters pass the whole info to the
importer). wgpu is pinned to the version the `cef` crate uses (29) and enabled
per-target for non-macOS only, so the macOS dylib does not pull it in.

## Oracle

`offscreen.rs` is the frozen reference (oracle), compiled only under the
`harness` feature. It is not the production path and must not be recycled as
one — an oracle reused as the implementation cannot verify anything.
`presenter/macos.rs` is the production copy that reproduces its algorithm; the
harness asserts the production output matches the oracle.

## Verification

- macOS and Linux compile locally: `cargo check --target <triple>`. Capture the
  exit code directly — `cargo check … | tail` reports the pipe's exit, not
  cargo's, and hides failures.
- Windows compiles in CI only. `cef-dll-sys` builds CEF's C++ wrapper with a
  resource compiler that is absent when cross-compiling from macOS. Linux is the
  local proxy for non-macOS code correctness.
- Native present is verified per-OS at runtime in CI (Linux under xvfb). A stub
  that only compiles is not a passing platform.

### Equivalence across platforms

Two planes, mirroring the terminal contract (canonical projection + oracle):

- Control plane, canonical: each presenter's frame-path decisions (surface
  scale, present coded size, colorspace, popup rect) project to a canonical form
  compared byte-exact across OS — the three presenters must make the same
  decisions.
- Data plane, fidelity: per OS, the presented surface equals the CEF source
  frame (the presenter is a pixel-preserving conduit). CEF guarantees the source
  is equivalent across OS.

## Status

- **macOS**: production present (raw Metal, `presenter/macos.rs`) is
  runtime-verified via the harness (frames presented + input, including IME).
- **Linux**: `presenter/linux.rs` — X11 child window under the parent XID
  (`x11-dl`), a `wgpu::Surface` on it, `osr_texture_import` → textured-quad
  render → present — is implemented and compiles clean for the linux target.
  On-screen rendering (child window mapped under the parent, frames visible) is
  verified in CI under xvfb, not by compilation.
- **Windows**: `presenter/windows.rs` (the same wgpu present with an HWND child
  window) is not yet written. It cannot be compiled from macOS — its CEF C++
  wrapper needs the Windows resource compiler — so it is authored and verified
  in CI.
- CEF loading on Linux/Windows (`libcef.{so,dll}`) is still stubbed; only the
  macOS `.framework` path is wired.

Both the accelerated (`on_accelerated_paint` → import → present) and the CPU
fallback (`on_paint` → upload → `present_cpu`) paths are wired for Linux; on
software-GL CI (lavapipe) CEF has no hardware DMA-BUF and takes the CPU path, so
that fallback is what makes frames appear in CI.

## Verification gates

| Scope | How it is verified | Where |
|---|---|---|
| macOS production present | harness renders a page; frames + input (incl. IME) | local, run the harness |
| macOS/Linux code correctness | `cargo check --target <triple>` (capture exit directly) | local |
| Windows code | `cargo check` (its CEF C++ wrapper needs the Windows RC) | CI only |
| On-screen render (Linux/Windows) | build + run the harness under xvfb, assert frames | CI only |
| Cross-OS equivalence | canonical control-plane record + per-OS data-plane fidelity | to build |

## Debugging

Run the engine without the app:

    cargo run --release --features harness --bin harness -- <dist-dir> offscreen

`make sidecar-chromium` (or `stage.sh <dist-dir>`) stages the CEF framework and
helper bundles. PASS (exit 0) means the cefQuery round-trip and, in offscreen
mode, the present path and input all worked. Frames are exposed as
`stats.dbg.framesPresented`; each presenter logs its first present once. Common
failures: on macOS, blank content usually means a missing `Helper (Renderer).app`
variant; on Linux, no frames usually means the child window was not mapped under
the parent, or the CPU fallback is not being exercised.

## Roadmap

- **A/B — done**: presenter interface + oracle split; crate un-gated; macOS and
  Linux compile clean.
- **C/D — Linux done, Windows in CI**: `presenter/linux.rs` implements the
  accelerated and CPU-fallback paths and compiles clean; `presenter/windows.rs`
  (HWND child, same wgpu pipeline) is authored and compiled in CI. On-screen is
  verified in CI under xvfb.
- **E**: CEF library loading for Linux/Windows (`libcef.{so,dll}`).
- **F**: the core hands the presenter a per-OS parent handle (X11 XID / HWND)
  next to the macOS NSView path; 5-target release-matrix CI.
- **Equivalence**: control-plane canonical projection compared across OS + a
  per-OS data-plane fidelity check.
