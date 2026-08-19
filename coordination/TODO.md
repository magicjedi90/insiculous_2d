# Task Queue - Insiculous 2D

## Web deployment is the open front (Aug 19 2026)

Studio premise pivot (Jesse, Aug 19 2026; adversarial review round in
`review/` this session): Be Insiculous is an **AI dev studio**; the six games
ship to the website **in their current look** (AI stand-ins allowed on the free
tier — tiered AI-art rule, DEION_STYLE.md §6). Phase H + I1/I2 lead; Phases F/G
are the parallel art track and land on the site as updates. The 20 Games
Challenge stays PAUSED at game 7 (Tetris). Full phase specs:
`PROJECT_ROADMAP.md` (incl. new Phase J — Insiculous Arcade, outline only).

(H1 spike + listen test PASSED Jul 30 2026 — audio decision FINAL: stay on
rodio. Firefox needs a full restart after enabling `dom.webgpu.enabled`, not
just a new tab. E3/E4 shipped Jul 30 — schema freeze REACHED; see PROGRESS.md.)

**Aug 19 2026: the Pong vertical slice SHIPPED** (H2 ☑, H3 ☑ revised to a
cfg-split — native loop unchanged, H4 ☑, H5 core ☑, H7 partial, H9 pong ☑
at 2.5 MiB, I1 staged awaiting Jesse's push) — see PROGRESS.md and the
updated roadmap Phase H table (incl. the detached-canvas / sRGB / 0-size
lessons for the next ports).

### Ready now — Phase H remainders (specs in PROJECT_ROADMAP.md)

- **TASK-H9 (remaining 5)** — Port snake → frogger → breakout →
  space_invaders → asteroids with the pong recipe: lib.rs/main.rs/
  `web_entry.rs` split (editor stays native-main-only), `[lib] cdylib+rlib`,
  wasm-target deps (`wasm-bindgen = "=0.2.126"`), `[profile.wasm-release]`,
  font path via `ctx.assets.base_path()`, then
  `scripts/build_wasm.sh <game_dir> <slug> --serve` + browser check.
  Breakout note: first game with scene RON files (loads via vfs — verify).
  Audio path loaders already work on wasm (vfs-routed, review-2 F1) — the
  first sound-using port still needs the H7 gesture gate to be audible.
- **TASK-H5 (remainder)** — `include_bytes!` bootstrap-minimum decision only
  (audio conversion landed via review-2 F1).
- **TASK-H6** — `KvStore` trait: native = JSON files (achievements keep atomic
  tmp+rename), wasm = localStorage; errors logged, never panic. (Web builds
  currently degrade to in-memory achievements + default bindings by setting
  no save paths.)
- **TASK-H7 (remaining)** — gesture-gated `OutputStream` init (start
  `disabled()` — already forced on wasm — upgrade on first gesture;
  `try_default()` Ok is NOT a health check).
- **TASK-H8** — Incremental wasm CI guard: `cargo check --target
  wasm32-unknown-unknown` starting on `common`/`ecs`, expanding crate-by-crate.
  Includes deciding the 3 renderer `arc_with_non_send_sync` wasm-clippy
  warnings (allow-lint vs restructure).

### Deploy

- **TASK-I1 (final step)** — Jesse pushes the staged insiculous_web changes
  (GameEmbed + pong.md + public/games/pong/v1/) to `milyramic/insiculous_web`
  `main` → Actions deploys → verify live at beinsiculous.com.
- **TASK-I2** — Remaining games on the site: same drop-in per game as H9
  builds land (page flip only when the build exists). The custom domain
  `beinsiculous.com` waits only on the Cloudflare zone + registrar
  nameservers (Jesse/Mily manual).

### Parallel art track (Phases F/G — no longer gating anything)

- **Phase F** — F1 style guide exists (`../games/deion_assets/DEION_STYLE.md`);
  F2 sync script, F3 `gen_tiles` offline generator (unblocks E5's `#rgba`
  error), F4 placeholder sheets, F5 first animated Deion.
- **Phase G** — re-skins Pong → Frogger → Breakout → Snake → Invaders →
  Asteroids; each finished re-skin ships to the site as an update.
- **TASK-E5 (remaining half)** — `#rgba` per-sprite save-error, flips ONLY
  after F3 migrates Frogger's tileset to PNGs.
- **TASK-E7** — Sprite-shader alpha-cutoff (configurable threshold); closes
  renderer TECH_DEBT alpha/depth item.
- **TASK-E8/E9** — Inspector wiring (`/add-component`) + docs.

**Instructions for agents:** Claim a task by creating `current_tasks/TASK-XXX.lock` with your agent ID and timestamp. Work the task, push, then remove the lock and move the task to PROGRESS.md.

**Priority order:** Work top-to-bottom. Higher tasks are higher priority.

---

## Task sourcing

Open technical debt is NOT duplicated here — it lives in the live docs
(root `TECH_DEBT.md` rollup → per-crate `TECH_DEBT.md` + `../games/TECH_DEBT.md`).
Engine feature gaps live in `PROJECT_ROADMAP.md` (now organized as Deion Pivot
Phases E–J). Pull from those when this queue is empty.

(Phase 1 editor task list and the Phase A/B queues shipped in full — see
`log_archive.md`.)
