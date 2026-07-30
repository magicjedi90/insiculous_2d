# TASK-H1 — WASM spike findings

**Date:** Jul 30 2026 · **Spike code:** `../spikes/h1_wasm/` (out-of-tree, never merged)
**Screenshot of the verified demo:** `../spikes/h1_wasm/h1_demo_verified.png`

## Verdict

**YES, with caveats.** Every dependency in the engine stack compiles for
`wasm32-unknown-unknown`, and a browser demo mirroring the engine's
architecture renders a fetched texture through wgpu 28 + winit 0.30 on WebGPU.
The audio question that motivated the spike is answered: **rodio 0.20 both
compiles for wasm32 and initialises a working output stream in a real
browser.** No dependency has to be dropped or feature-gated away for the web.

The caveats are all engine-side, not dependency-side: `pollster::block_on`,
`std::thread::sleep`, `std::time::Instant`, and `std::fs` each need a web path
(details in "Gotchas for H2–H6"). None is architecturally hard; together they
are the real cost of Phase H.

**Unverified by agent: audible sound.** The wrapper reports itself enabled and
the sound decoded, but nobody has heard it. Jesse must confirm — see "How to
run". This is the one check an agent cannot do.

## What was verified, and how

| Step | Result | Evidence |
|---|---|---|
| 1. wgpu 28 + winit 0.30 render in browser | **PASS** | Screenshot: sprite on canvas, `gfx: ready` |
| 2. Texture fetched over HTTP → `image` decode → GPU | **PASS** | `texture: 242 bytes fetched, decoded 32x32`; server log shows the browser GET for `deion.png` |
| 3. rodio audio wrapper initialises on wasm | **PASS** | `enabled — 1 sound loaded (handle 1)` — `OutputStream::try_default()` returned Ok in Firefox |
| 3b. Sound is *audible* and volume-respecting | **UNVERIFIED** | needs a human |
| 4. Per-dependency wasm probes | **PASS (14/14)** | table below |
| 5. Browser screenshot | **PASS** | windowed Firefox on an RTX 5080 |

Rendering was verified in **windowed** Firefox. Headless Firefox is useless
here: `navigator.gpu` is absent by default, and even with the pref forced on,
`requestAdapter()` never resolves without a GPU-backed compositor. If CI ever
needs to smoke-test the web build, it needs a real GPU session or headless
Chrome with `--enable-unsafe-swiftshader`, not headless Firefox.

## Per-dependency pass/fail + exact feature sets (finding F3)

All probed with `cargo check --target wasm32-unknown-unknown`, rustc 1.94.0.
**Every probe passed.** The "Cargo.toml for wasm" column is the known-good
configuration H2 should start from.

| Dependency | Engine version | wasm32 | Cargo.toml for wasm | Notes |
|---|---|---|---|---|
| `glam` | 0.30.10 | PASS | `features = ["serde"]` — unchanged | pure Rust |
| `serde` / `serde_json` / `ron` / `toml` | 1.0 / 1.0 / 0.12 / 0.9.11 | PASS | unchanged | pure Rust |
| `fontdue` | 0.9 | PASS | unchanged (default) | keep `parallel` off |
| `image` | 0.25 | PASS | `default-features = false, features = ["png","jpeg","bmp","gif"]` — the engine's exact set, verbatim | |
| `rapier2d` | 0.23 | PASS | unchanged (default `dim2` + `f32`) | already depends on `web-time`; keep `parallel`/`rayon` off |
| `wgpu` | 28.0 | PASS | works **unchanged**; leaner is `default-features = false, features = ["webgpu","wgsl","std"]` | `std` is a real feature in wgpu 28 — omitting it breaks the build |
| `winit` | 0.30 | PASS | unchanged (`features = ["serde"]`) | web support is automatic on the wasm32 target; no feature flag |
| `gilrs` | 0.11 | PASS | unchanged (default) | **the roadmap's "feature-gate gilrs to a no-op" is unnecessary** — gilrs-core ships a web-sys Gamepad backend |
| `rodio` | 0.20 | PASS | `default-features = false, features = ["symphonia-all", "wasm-bindgen"]` | see the audio section |
| `web-time` | 1.1 (new) | PASS | `web-time = "1.1"` | drop-in `Instant` + `SystemTime` |

Two extra probes worth recording: rodio compiles on wasm32 **with and without**
the `wasm-bindgen` feature, and the full API surface (`OutputStream`, `Sink`,
`Decoder`, `Source::repeat_infinite`) compiles in both configurations. Keep the
feature on anyway — it enables cpal's wasm-bindgen glue, and it is what the
runtime-verified build used.

## Web audio backend decision: **rodio** (no change)

**Decision: stay on rodio 0.20.** kira and the web-sys `AudioContext` shim were
not needed and should not be pursued.

Evidence, strongest first:

1. **Runtime, not just compile-time.** In Firefox the wrapper's
   `new_or_disabled()` reported `enabled`, meaning
   `OutputStream::try_default()` returned `Ok` and cpal actually constructed an
   `AudioContext`. This was the genuine risk — compilation was never really in
   doubt once cpal's backend was confirmed.
2. **The decode path works on wasm.** `load_sound_from_bytes` decoded a
   programmatically generated WAV through symphonia and issued handle 1.
3. **cpal 0.15.3 has a first-class wasm32 backend** — non-optional `js-sys` and
   `web-sys` deps (`AudioContext`, `AudioBufferSourceNode`,
   `AudioDestinationNode`, `AudioContextState`) gated on
   `cfg(all(target_arch = "wasm32", target_os = "unknown"))`.
4. **The whole engine surface maps over.** `../spikes/h1_wasm/src/audio.rs`
   mirrors `crates/audio/src/manager.rs`: `new_or_disabled` that never panics,
   disabled-mode that still validates decodes and no-ops playback,
   `load_sound_from_bytes`, `play_with_settings` with
   `base * sfx * master` applied at the sink, music loop/stop, and bus changes
   re-applied to live sinks. It compiles for wasm32 with zero warnings.

**One real API change is forced.** The engine's `load_sound` and `play_music`
take `AsRef<Path>` and hit `std::fs`. There is no filesystem in a browser, so
the web mirror is bytes-only (`play_music_from_bytes`). H2 should make the
byte-taking entry points the primary API on all platforms and let the path
versions be thin native-only conveniences — otherwise every call site forks.

### Autoplay policy

Browsers refuse to start audio without a user gesture. The rule that matters:
**construct the rodio `OutputStream` inside a gesture handler, not at startup.**
An `AudioContext` created before any gesture begins in the `suspended` state and
stays silent even after the user interacts.

For the engine this means `AudioManager` cannot be built eagerly in
`GameRunner`. Options, in order of preference: (a) construct it lazily on the
first input event, or (b) keep the engine's existing `disabled()` mode as the
startup state and upgrade to a real device on first gesture — the engine
already has that fallback, so (b) is close to free.

Worth noting: the spike's no-gesture probe reported `enabled` anyway, so
`try_default()` succeeding does **not** prove the context is running. Do not
use it as a health check.

## Asset fetch notes

`fetch(url) → ArrayBuffer → Vec<u8> → image::load_from_memory → queue.write_texture`
works exactly as hoped (`fetch_bytes` in `src/lib.rs`). Points for H4:

- Asset loading becomes **async**, and that is the infectious part — `init_gfx`
  had to be an async task, which is what forces the `Option<State>` restructure
  below. Scene loading, fonts, locales, and sounds all inherit this.
- Relative URLs resolve against the page, so `assets/...` paths work unchanged
  if the deploy layout mirrors the repo layout.
- Nothing needs a special MIME type beyond `.wasm` → `application/wasm`, which
  `python3 -m http.server` already sends. GitHub Pages and itch.io both do too.
- Errors are HTTP errors: the loader must handle 404 as a normal failure rather
  than the `io::Error` shape the engine expects today.

## Gotchas for H2–H6

Concrete engine blockers, all found by grepping `crates/`:

1. **`pollster::block_on` cannot work on wasm.** `render_manager.rs:72` blocks
   on `renderer::init_with_config`. Renderer init must become async, which
   cascades: `GameRunner` cannot hold a live renderer until the future
   resolves, so it needs an `Option<Renderer>` filled in later. The spike shows
   the shape — `App::pending: Rc<RefCell<Option<Gfx>>>`, populated by
   `spawn_local` and drained on the next event.
2. **`std::thread::sleep` in the FPS throttle.** `game_loop_manager.rs:56`.
   Must compile out on wasm; the browser paces frames via
   `requestAnimationFrame` and `GameConfig.target_fps` becomes meaningless
   there. Do not silently ignore it — document that the knob is native-only.
3. **`std::time::Instant` / `SystemTime` panic on wasm.** Used in
   `game_loop_manager.rs`, `timing.rs`, and `achievements/mod.rs`. `web-time`
   is a drop-in for both; the cleanest fix is a `common` time alias that
   re-exports `std::time` natively and `web_time` on wasm, so no call site
   changes.
4. **`std::fs` has no web equivalent.** Present in `scene_loader.rs`,
   `localization.rs`, `input_settings_io.rs`, `achievements/mod.rs`,
   `ui/font/mod.rs`, and `audio/manager.rs`. Reads become fetches; **writes**
   (achievement saves, input settings) need `localStorage`, and the engine's
   atomic temp-file-plus-rename save has no analogue there.
   *Added post-spike by E4 (Jul 30):* `assets/sprite_sheet.rs` —
   `prepare_sheet` (`std::fs::read_to_string` + `image::image_dimensions` on
   a path) and `SidecarCache::read` (`Path::exists` probe), now on the load
   path of every file-referencing scene texture. H2 needs a bytes-primary
   redesign here too (dimension probe from fetched bytes, not a path).
5. **winit does not put its canvas in the DOM.** `spawn_app` creates the canvas
   but you must append it yourself via `WindowExtWebSys::canvas()`. Easy to
   miss — everything "works" and nothing appears.
6. **`spawn_app` returns immediately** and takes ownership of the app. Anything
   the caller wanted to do after `run()` has to move into the handler.
   `resumed` can fire more than once; guard it (the spike checks
   `self.window.is_some()`).
7. **Redraw-driven loop.** `RedrawRequested` → draw → `request_redraw()` maps
   onto `requestAnimationFrame`. Do not drive frames from `about_to_wait`.
8. **wasm-bindgen CLI must match the crate version exactly** (here 0.2.126).
   Pin the crate with `=` and install the CLI at the same version, or builds
   fail with confusing schema errors.
9. **WebGPU is not on by default in Firefox 153 on Linux** — `navigator.gpu` is
   absent until `dom.webgpu.enabled=true`. Since the roadmap is WebGPU-only,
   the page must feature-detect and show a real message rather than a blank
   canvas. Chrome/Edge desktop are fine.
10. **Binary size.** The spike is a bare triangle-plus-audio and is already
    **1.83 MB** of wasm after `wasm-bindgen` (3.07 MB before) at
    `opt-level = "s"`. The real engine will be considerably larger. Budget for
    `wasm-opt -Oz` and gzip/brotli on the host in H6; `wasm-opt` is **not**
    installed on this machine.

## Toolchain state (what had to be installed)

- `rustup target add wasm32-unknown-unknown` — **was not installed**
- `cargo install wasm-bindgen-cli --version 0.2.126 --locked` — **was not installed**
- Not installed and not required: `trunk`, `wasm-pack`, `wasm-opt` (will be
  wanted in H6)
- rustc/cargo 1.94.0 · Firefox 153 (snap) · RTX 5080

Two environment quirks that cost time and will bite the next agent:
**Firefox is a snap**, so it cannot read a profile or write a screenshot under
`/tmp` — use paths under `$HOME`. And `firefox --screenshot` always runs
headless and fires at the `load` event, so it captures before any async init;
the spike works around it with a deliberately slow hidden image
(`tools/shot_server.py`).

## How to run the demo

```bash
cd ../spikes/h1_wasm
./build.sh            # builds, runs wasm-bindgen, regenerates the PNG, serves :8777
```

Then open **http://127.0.0.1:8777/** — in Chrome it works as-is; in Firefox set
`dom.webgpu.enabled=true` in `about:config` first.

Expected: a pixel-art sprite pulsing on the canvas, `gfx: ready`, and
`texture: 242 bytes fetched, decoded 32x32`.

**Jesse — the audio check (the one thing an agent cannot verify):** click
**Enable Audio & Start**, then

1. **SFX @ 1.00** then **SFX @ 0.25** — the second must be clearly quieter
   (proves `base * sfx * master` at the sink).
2. **Music loop** — a 2-second arpeggio should loop seamlessly.
3. With music playing, **master 0.20** — the *already-playing* music must duck
   immediately (proves bus changes re-apply to live sinks, like the engine).
4. **Stop music** — silence.

If all four behave, the audio backend decision is confirmed and H2 can proceed
on rodio. If any fails, the fallback ladder is kira first, then a web-sys
`AudioContext` shim behind the same wrapper surface.

## Nothing blocks Phase H as roadmapped

No dependency has to be replaced. The WebGPU-only decision holds. The work in
H2–H6 is the async/`web-time`/fetch/localStorage refactor listed above, plus a
binary-size pass — all known quantities, none of them research.
