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

### Ready now — Phase H (parallelizable across crates; specs in PROJECT_ROADMAP.md)

- **TASK-H2** — `web-time` swap: replaces `Instant`/`SystemTime` in
  game_loop_manager, timing, lifecycle, achievements.
- **TASK-H3** — Redraw-driven loop: `RedrawRequested` + `request_redraw` (one
  model native+web); `thread::sleep` throttle native-only.
- **TASK-H4** — Async renderer init: `wasm_bindgen_futures::spawn_local` on
  wasm; pollster only at the native outer edge. SINGLE-AGENT.
- **TASK-H5** — Asset manifest + fetch boot phase: generated per-game manifest;
  web boot fetches all entries into a bytes map; loaders get bytes-primary
  twins (`load_sound`/`play_music` become bytes-primary per H1 finding).
- **TASK-H6** — `KvStore` trait: native = JSON files (achievements keep atomic
  tmp+rename), wasm = localStorage; errors logged, never panic.
- **TASK-H7 (remaining)** — gesture-gated `OutputStream` init (start
  `disabled()`, upgrade on first gesture; `try_default()` Ok is NOT a health
  check).
- **TASK-H8** — Incremental wasm CI guard: `cargo check --target
  wasm32-unknown-unknown` starting on `common`/`ecs`, expanding crate-by-crate.

### Sequenced after H2–H8

- **TASK-H9** — Port all 6 games: shared `scripts/build_wasm.sh` + index.html
  template + release-profile snippet. Does NOT gate on Phase G — current look
  ships.
- **TASK-I1/I2** — Games live on the site (`../insiculous_web/`, Cloudflare
  Workers, drop-in `public/games/<slug>/v1/` convention; flip a page to
  `playable` only when its build lands). Deploys already run via GitHub
  Actions on Mily's repo (`milyramic/insiculous_web`); the custom domain
  `beinsiculous.com` is configured and waits only on the Cloudflare zone +
  registrar nameservers (Jesse/Mily manual).

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
