# Technical Debt: renderer

Last audited: January 2026

## Summary
- DRY violations: 3 (1 resolved)
- SRP violations: 3 (1 documented)
- KISS violations: 0 (1 resolved)
- Architecture issues: 4 (1 resolved)

---

## DRY Violations

### [DRY-001] Duplicate surface error handling in renderer.rs
- **File:** `renderer.rs`
- **Lines:** 127-142 (render), 219-234 (render_with_sprites_internal)
- **Issue:** Surface error handling is duplicated in both render methods with identical logic:
  ```rust
  match self.surface.get_current_texture() {
      Ok(frame) => frame,
      Err(wgpu::SurfaceError::Lost) => { return Err(RendererError::SurfaceError(...)); }
      Err(wgpu::SurfaceError::OutOfMemory) => { return Err(RendererError::RenderingError(...)); }
      Err(e) => { log::warn!(...); return Ok(()); }
  }
  ```
- **Suggested fix:** Extract to `fn acquire_frame(&self) -> Result<wgpu::SurfaceTexture, RendererError>`.
- **Priority:** Medium

### ~~[DRY-002] Duplicate sampler creation in multiple locations~~ ✅ RESOLVED
- **File:** `sprite.rs`, `sprite_data.rs`, `texture.rs`
- **Resolution:** Added `SamplerConfig::create_sampler(&self, device, label)` method to `texture.rs`. All 4 locations now delegate to this shared helper:
  - `SpritePipeline::new()` → `SamplerConfig::default().create_sampler(device, Some("Sprite Sampler"))`
  - `TextureAtlas::new()` → `SamplerConfig::default().create_sampler(device, Some("Texture Atlas Sampler"))`
  - `TextureResource::new()` → `SamplerConfig::default().create_sampler(device, Some("Texture Sampler"))`
  - `TextureManager::create_sampler()` → `config.create_sampler(&self.device, Some("Texture Sampler"))`

### [DRY-003] Duplicate render pass descriptor in sprite.rs
- **File:** `sprite.rs`
- **Lines:** 521-536
- **Issue:** The `RenderPassDescriptor` creation with color attachment setup is similar to what's in `renderer.rs:157-173`. Both configure the same operations.
- **Suggested fix:** Consider a helper function or builder for creating standard render pass descriptors.
- **Priority:** Low

### [DRY-004] Duplicate texture descriptor creation
- **File:** `renderer.rs:338-351`, `sprite.rs:630-643`, `texture.rs:316-333`
- **Issue:** Similar `TextureDescriptor` patterns for creating 2D textures with RGBA8UnormSrgb format.
- **Suggested fix:** Create a helper `create_texture_2d(device, width, height, label)` that encapsulates common texture creation.
- **Priority:** Low

---

## SRP Violations

### [SRP-001] SpritePipeline holds too many GPU resources
- **File:** `sprite.rs`
- **Lines:** 225-254
- **Issue:** `SpritePipeline` manages 13 different concerns:
  1. Render pipeline
  2. Pipeline layout
  3. Vertex buffer
  4. Instance buffer (DynamicBuffer)
  5. Index buffer
  6. Camera uniform buffer
  7. Texture bind group layout
  8. Camera bind group layout
  9. Camera bind group
  10. Texture bind group cache
  11. Sampler
  12. Max sprites config
  13. Device reference

  This makes the struct difficult to test, maintain, and extend.
- **Suggested fix (from ANALYSIS.md):** Split into:
  - `PipelineResources` - Render pipeline and layouts
  - `BufferManager` - Vertex, instance, index buffers
  - `CameraManager` - Camera uniform and bind group
  - `TextureBindGroupManager` - Texture bind groups and cache
- **Priority:** Medium

### ~~[SRP-002] Renderer handles both initialization and rendering~~ ✅ DOCUMENTED
- **File:** `renderer.rs`
- **Resolution:** Added module-level documentation explaining the intentional design:
  - Initialization and rendering are tightly coupled in WGPU (surface, device, queue share lifetimes)
  - Splitting would add complexity without clear benefit for a 2D game engine
  - `run_with_app` is noted as legacy - new code should use `Game` trait and `run_game()`
- **Resolved:** January 2026

### [SRP-003] RenderPipelineInspector mixes logging with operation wrapping
- **File:** `render_pipeline_inspector.rs`
- **Lines:** 44-322
- **Issue:** The inspector class handles:
  1. Configuration management (logging, validation, timing flags)
  2. Operation logging
  3. Operation wrapping (inspect_* methods)
  4. Report generation
  5. History management

  The `InspectedRenderPass` also duplicates render pass functionality.
- **Suggested fix:** Split into `RenderLogger` (logging/history) and `RenderInspector` (wrapping/validation).
- **Priority:** Low (debugging utility, not critical path)

---

## KISS Violations

### [KISS-001] RenderPipelineInspector is over-engineered for debugging
- **File:** `render_pipeline_inspector.rs`
- **Lines:** 1-435
- **Issue:** This 435-line debugging utility provides:
  - Arc<Mutex<>> thread-safe operation history
  - Detailed timing instrumentation
  - Multiple inspection methods for different operations
  - Report generation
  - Custom `InspectedRenderPass` wrapper

  For a debugging tool, this is significant overhead. The tool logs every GPU operation but isn't used in the main render path.
- **Suggested fix:** Either integrate into the main render path or simplify to basic logging. Consider using `tracing` crate with spans instead.
- **Priority:** Low (not in critical path)

### ~~[KISS-002] Unsafe transmute for surface lifetime~~ ✅ RESOLVED
- **File:** `renderer.rs`
- **Resolution:** Removed unsafe transmute. WGPU 28.0.0 properly infers `'static` lifetime when `Arc<Window>` is passed to `create_surface()`. The fix uses explicit type annotation `let surface: Surface<'static> = ...` to help the compiler infer the correct lifetime safely.

---

## Architecture Issues

### [ARCH-001] Redundant device/queue accessors
- **File:** `renderer.rs`
- **Lines:** 269-286
- **Issue (from ANALYSIS.md):** Both Arc-returning and borrowed versions exist:
  ```rust
  pub fn device(&self) -> Arc<Device>     // Returns clone of Arc
  pub fn device_ref(&self) -> &Device     // Returns reference
  pub fn queue(&self) -> Arc<Queue>       // Returns clone of Arc
  pub fn queue_ref(&self) -> &Queue       // Returns reference
  ```
  This creates API confusion - callers don't know which to use.
- **Suggested fix:** Keep only the borrowed accessors (`device_ref`, `queue_ref`). Callers who need ownership can access the Arc through a single method.
- **Priority:** Medium

### [ARCH-002] Time struct in renderer crate is misplaced
- **File:** `lib.rs`
- **Lines:** 34-50
- **Issue:** The `Time` struct (delta_seconds, elapsed_seconds) is defined in the renderer crate but has nothing to do with rendering. It's a general game timing concept.
- **Suggested fix:** Move to `common` crate or `engine_core` crate where timing concepts belong.
- **Priority:** Low

### [ARCH-003] Dead code with #[allow(dead_code)] suppressions
- **Files:** Multiple
- **Issue:** Several dead code suppressions exist:
  - `sprite.rs:164`: `max_sprites_per_batch` in SpriteBatcher (field stored but never read)
  - `sprite.rs:228`: `layout` in SpritePipeline (created for pipeline, not used after)
  - `sprite.rs:247`: `sampler` in SpritePipeline (stored but accessed via bind group)
  - `sprite_data.rs:220-221`: `usage` in DynamicBuffer (stored for potential recreation)
  - `texture.rs:363`: `create_placeholder_texture` method (reserved for future)
- **Suggested fix:** Either use these fields/methods or remove them. The "potential future use" justification adds maintenance burden.
- **Priority:** Low

### ~~[ARCH-004] Inconsistent error types between modules~~ ✅ RESOLVED
- **Files:** `error.rs`, `texture.rs`
- **Resolution:** Added `From<TextureError>` implementation for `RendererError`:
  ```rust
  #[error("Texture error: {0}")]
  TextureError(#[from] TextureError),
  ```
  Callers can now use `?` to automatically convert `TextureError` to `RendererError` when needed.
- **Resolved:** January 2026

---

## Previously Resolved (Reference)

These issues have been resolved:

| Issue | Resolution |
|-------|------------|
| Bind groups created every frame | FIXED: Camera bind group cached, texture bind groups cached per handle |
| ECS sprites using wrong texture handle | FIXED: Now uses `ecs_sprite.texture_handle` correctly |
| Missing `prepare_sprites()` call | FIXED: Called before drawing |
| KISS-002: Unsafe transmute | FIXED: Removed transmute, WGPU 28.0.0 infers `'static` from `Arc<Window>` |

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 9 |
| Total lines | ~2,600 |
| Test coverage | 62 tests (100% pass rate) |
| `#[allow(dead_code)]` | 5 instances |
| `unsafe` blocks | 0 |
| High priority issues | 0 |
| Medium priority issues | 3 |
| Low priority issues | 7 |

---

## Recommendations

### ✅ Completed
1. ~~**Fix DRY-001** - Extract surface acquisition helper to reduce duplication~~ (done: acquire_frame helper)
2. ~~**Fix DRY-002** - Create shared sampler creation helper~~ (done: SamplerConfig::create_sampler)
3. ~~**Fix ARCH-001** - Consolidate device/queue accessors~~ (documented: both versions kept for different use cases)
4. ~~**Fix ARCH-002** - Move Time struct to appropriate crate~~ (done: moved to common crate)
5. ~~**Fix KISS-002** - Remove unsafe transmute~~ (done: WGPU 28.0.0 properly infers lifetime)

### Short-term Improvements
1. **Fix SRP-001** - Split SpritePipeline into focused structs

### Technical Debt Backlog
- ARCH-003: Review and remove dead code
- ARCH-004: Unify error types
- DRY-003: Consolidate render pass descriptor creation
- DRY-004: Create texture descriptor helper

---

## Cross-Reference with PROJECT_ROADMAP.md / ANALYSIS.md

| This Report | ANALYSIS.md | Status |
|-------------|-------------|--------|
| SRP-001: SpritePipeline too large | "Split SpritePipeline into focused structs" | Known, unresolved |
| ARCH-001: Redundant accessors | "Consolidate device/queue accessors" | ✅ Documented (both versions kept intentionally) |
| DRY-001: Surface error handling | Not tracked | ✅ RESOLVED (acquire_frame helper) |
| DRY-002: Sampler creation | Not tracked | ✅ RESOLVED (SamplerConfig::create_sampler) |
| KISS-002: Unsafe transmute | Not tracked | ✅ RESOLVED |
| ARCH-002: Time misplaced | Not tracked | ✅ RESOLVED (moved to common crate)
