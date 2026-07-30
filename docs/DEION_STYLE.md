# DEION STYLE — The Insiculous World Bible & Asset Guide

**Status: DRAFT (F1) — castings and palette are PROPOSALS awaiting Jesse's
sign-off.** Sprite *frame* art may begin immediately per the metrics below.
Sheet *layouts* are provisional until the `.sheet.ron` schema freezes at the
E2+E4 checkpoint (see `PROJECT_ROADMAP.md`, Phase E) — draw frames now,
assemble sheets later.

---

## 1. World Bible

**The world is food-coded.** Every arena, obstacle, enemy, and prop reads as
food or kitchen: countertop battlegrounds, soup rivers, cake-brick fortresses,
descending bread squadrons. Deion and Cubert are the *non-food* things in it —
water and ice in a world of snacks — which is exactly why they pop on screen.

Tone: playful arcade menace. Food is charming AND out to get you. The
"insiculous" (insanely-ridiculous) chaos tiers push the food world from
appetizing (Normal) to feral (Insiculous).

### The Cast (canon — Jesse's characters)

**Deion the Insiculous** — the hero. A ball of DEIONized water with an icicle
mohawk. Concept reference: `docs/concepts/deion_ref.png` (64×64 canvas, from
`~/Pictures/sprites/Deion.aseprite`). He is literally water: he bounces,
splashes, drips, and refreezes.
- Body: light-blue water ball, dark outline, bright rim highlight, simple
  wide eyes (per the reference).
- Mohawk: icicle spikes, white/pale-cyan, always visible in silhouette.
- **Projectiles: icicle spikes fired from his mohawk** — this is the universal
  projectile language wherever Deion shoots.
- Poses: idle = gentle wobble/slosh; walk = rolling bounce; jump = stretch up,
  drip; hurt = splash-flatten, mohawk droops.

**Cubert** — Deion's best friend, an ice cube. The Diddy to his Kong, the
Luigi to his Mario. **Default P2 character across all 2-player modes.**
- Square silhouette (contrast to Deion's round one), same ice-family palette,
  pale-cyan faces with frost sparkle.
- Physical comedy is his thing: he slides, he slips, he falls over on
  collision. Signature clips: `slip` (feet-out pratfall) and `slide`.
- Projectile flavor when he shoots: ice chips/frost shards (smaller, scrappier
  than Deion's icicles).

**Dr. Maxwell** — the big bad, Deion's arch-rival and nemesis. A devil's food
cake with horns.
- Dark chocolate-cake body (near-black browns), frosting details, small
  glowing horns (candy-red ramp). Menacing but still a cake.
- Appears as boss/UFO/fortress-owner across games — the recurring villain.

**In-Bread Yokels** — Dr. Maxwell's rank-and-file: toast slices with a fried
egg in the middle. Crust-brown ramp + egg-white/yolk (cream + cheese ramps).
The egg is the face. Dopey, numerous, march in formation.

**The Flying Funk** — Deion's spaceship (easter egg: named after a song by
Jesse's friend). The name appears on the hull/HUD wherever the ship shows up.

## 2. Sprite Metrics (settled — not up for reinterpretation)

| Rule | Value |
|------|-------|
| Base cell | **16×16 px** |
| Larger subjects | Multiples of the cell on the 16px grid (32×32 = 2×2 cells) |
| Scale in engine | **5× integer** → 16px cell = `RENDER_UNIT` (80) = 1 world unit = 1 collider unit |
| Filtering | **Nearest** (declared in `.sheet.ron`; sheet loads default Nearest) |
| Rotation | Prefer drawn rotations/poses over runtime rotation for characters; runtime rotation OK for projectiles/debris |

Physics ignores `Transform2D.scale` — the 1:1 cell/unit mapping is what keeps
sprites and colliders aligned. Never size a sprite via `scale` to fake a
different footprint; draw it at the right cell size instead.

## 3. Export Rules (Aseprite → PNG)

1. **No anti-aliased or semi-transparent edges.** Pixels are fully opaque or
   fully transparent (indexed-color mode gives this for free). The renderer
   uses alpha-cutoff; soft edges will fringe or vanish.
2. Plain **PNG**, transparent background, no embedded color profile weirdness.
3. Work in indexed mode against the palette ramps below; new colors need a
   palette-section update in this doc, not ad-hoc picks.
4. One subject per sheet file. Don't pack unrelated subjects together.

## 4. Palette (PROPOSAL — SNES-style limited ramps)

Universal outline for everything: **`#14101F`** (near-black plum — warmer than
pure black, reads SNES).

**Deion / water & ice (hero-exclusive — nothing else uses this ramp):**
| Role | Hex |
|------|-----|
| Deep water shadow | `#1B2A52` |
| Core water | `#2E62C9` |
| Lit water | `#4FA4E8` |
| Surface sparkle | `#9FDCF2` |
| Rim highlight / icicle | `#E8FBFF` |

**Food warm ramps (world, enemies, props):**
| Material | Shadow → Highlight |
|----------|--------------------|
| Crust / bread / wood-grain browns | `#5C3A24` → `#9C6B3C` → `#D8A05C` → `#F2D6A0` |
| Candy / sauce reds | `#8A1E30` → `#D33F4C` → `#F4796F` → `#FFB3A0` |
| Cheese / butter / fry yellows | `#B77818` → `#E8B23C` → `#FFE08A` → `#FFF4C4` |
| Veggie / lime greens | `#1E5C3A` → `#3FA45C` → `#8CD98C` → `#D6F5C9` |
| Grape / berry purples | `#3D2352` → `#6E3E9C` → `#A66FD4` → `#DCC0F2` |
| Cream / plate neutrals | `#4A4458` → `#8C86A0` → `#CFC9DC` → `#F5F2FA` |
| Devil's-food darks (Dr. Maxwell) | `#241418` → `#3A211E` → `#59322B` → `#7A4A3A` |

Rule of thumb: **max 4 shades per material + the universal outline**; a 16px
sprite should rarely use more than 2 ramps.

**Character → ramp mapping:** Deion + Cubert own the water/ice family
(nothing food-side may use it). Dr. Maxwell = devil's-food darks + candy-red
horns/eyes. In-Bread Yokels = crust browns (toast) + cream ramp (egg white) +
cheese ramp (yolk). Flying Funk hull = plate neutrals with an ice-family
canopy.

### ChaosTheme = the FX/accent layer (mapping of record)
The neon `ChaosTheme` palette survives, demoted from "the art" to "the
electricity around the art":

| ChaosTheme token | New role |
|------------------|----------|
| `bg_color` | Ambient backdrop behind the food world (per-game) |
| `structure_color` | Debug/wireframe overlays only (no longer gameplay art) |
| `accent_color` | Bloom FX: particles, pickups flashes, chaos-mode auras |
| `grid_color` | Optional background grid, per game's taste |
| `banner_text` / `banner_color` | Chaos-mode HUD banner (unchanged) |
| `particle_count_mult` | Unchanged |

Sprites themselves stay in the food palette; `emissive` is reserved for FX
moments (power-ups, explosions, chaos escalation), not resting states.

## 5. Castings (PROPOSAL — react, veto, recast freely)

| Game | Player(s) | World / obstacles | Enemies / hazards | Notes |
|------|-----------|-------------------|-------------------|-------|
| **Pong** | Deion IS the ball. Paddles = crusty **baguettes**; the AI opponent is an **In-Bread Yokel** wielding his paddle (2P human = Cubert wields the other) | Countertop court; crumb particles on hit | — | Deion squash-stretches on paddle hits; icicle mohawk trails |
| **Breakout** | Paddle = **butter-pat on toast** (co-op: P2 = Cubert's frost-toast paddle); ball = Deion | **Dr. Maxwell's devil's-food fortress** — brick wall = cake segments with frosting mortar; armored bricks = foil-wrapped squares (foil tears per hit) | Falling pickups = **sprinkles / candy capsules**; Maxwell taunt cameo on level clear | Wrecking-ball mode = Deion frozen solid (ice ball sprite) |
| **Space Invaders** | P1 Deion firing **mohawk icicles**; P2 Cubert firing **ice chips** | Bunkers = **burger buns** (bites taken as they degrade) | Descending ranks of **In-Bread Yokels** (2–3 toast designs, egg faces); UFO flyby = **Dr. Maxwell** on a cake saucer | Yokels do a march-wiggle per step |
| **Snake** | **Cubert collecting ice cubes** — each cube joins the train behind him. Death = his signature **slip-and-fall** on collision | Kitchen-floor tile arena | Pellet = **ice cube** (from a leaky freezer); versus = the other trail | Versus proposal: P1 Cubert (cube trail) vs P2 Deion (water-droplet trail) — or two Cuberts, palette-shifted |
| **Asteroids** | Deion piloting the **Flying Funk** (name painted on the hull — easter egg); P2 co-op = Cubert's ship (name TBD by Jesse) | Asteroids = tumbling **popcorn chunks** (3 sizes) | UFO = **Dr. Maxwell's cake saucer** | Ship shots = icicle spikes. Drawn rotation set (16 angles) vs runtime rotation — decide at E4 |
| **Frogger** | Deion **hopping home to the ice-cube tray** (home slots = tray sockets, filling with a frozen Deion each); P2 = Cubert | Road lanes = **conveyor belts of rolling food carts** (sushi rolls, hot dogs, donuts); river = **soup**; logs = **celery sticks / baguettes** | Crocs = **snapping hot-dog buns**; turtles = **crackers** that sink into the soup | Thematically the anchor game: water-guy crossing food traffic to reach the freezer |

Connective tissue across all games: Deion (or Cubert) as the player,
Dr. Maxwell's forces as the opposition, In-Bread Yokels as the rank-and-file,
icicle projectiles, and the Flying Funk wherever a ship is called for.

## 6. File & Naming Conventions

```
docs/concepts/                    # reference/concept art (not shipped)
  deion_ref.png
../games/deion_assets/            # canonical source (synced per game, F2)
  characters/deion/deion_16.png       + deion_16.sheet.ron
  characters/cubert/cubert_16.png
  characters/maxwell/maxwell_32.png
  characters/yokel/yokel_16.png
  ships/flying_funk_32.png
  tiles/<set>_16.png                  # gen_tiles output lands here too
  props/<prop>_16.png
```

- Files: `snake_case`, suffix `_16`/`_32` = cell size of the grid.
- Every sheet PNG ships with a same-stem `.sheet.ron` sidecar (schema lands in
  E4; agents write sidecars, not Jesse).
- **Clips are the stable API**: game code references clip *names* only —
  `idle`, `walk`, `jump`, `hurt`, `attack`, `die` (lowercase snake_case).
  Grid positions may be rearranged freely as long as names survive.
- Placeholder sheets (F4) use final clip names + final cell sizes with
  blocked-out colors — swapping in real art must never touch game code.

## 7. Responsibility Split

**Jesse (hand-drawn, Aseprite):**
- Deion hero sheet: `idle`, `walk`, `jump`, `hurt` minimum (+ per-game
  variants: frozen ball, hopping) — building on `docs/concepts/deion_ref.png`.
- Cubert sheet: `idle`, `walk`/`slide`, `slip` (the pratfall), `hurt`.
- Dr. Maxwell (at least a menacing `idle` + saucer/fortress appearances).
- In-Bread Yokel (1–3 toast variants, `march` clip).
- The Flying Funk (+ Cubert's ship if co-op ships differ; name Cubert's ship).
- Palette + castings sign-off (edit sections 4/5 in place — this doc is the
  single source of truth for style decisions).

**Agents (generated / tooling):**
- All tiles & simple props via `scripts/gen_tiles` (offline PNGs — never
  runtime rgba textures).
- All `.sheet.ron` sidecars, the `deion_assets` sync script (+ `--check`),
  placeholder sheets for all 6 games.
- Keeping this doc, `/new-game`, and `training.md` in sync as phases land.

## 8. Animation Vocabulary (baseline; per-clip fps in the sidecar)

| Clip | Frames (typical) | Notes |
|------|------------------|-------|
| `idle` | 2–4 | Slow slosh/wobble, ~4 fps |
| `walk` | 4–6 | Rolling bounce, ~10 fps |
| `jump` | 2–3 | Stretch-up / apex / fall (non-looping) |
| `hurt` | 2 | Splash-flatten (non-looping) |
| `slip` (Cubert) | 3–4 | Feet-out pratfall, non-looping — his collision/death read |
| `slide` (Cubert) | 2 | Frictionless glide loop |
| enemy `march`/`roll` | 2–4 | Yokel wiggle, cart roll |

Non-looping clips end on their last frame (`is_complete` drives game logic —
respawn, invuln flash, etc.).
