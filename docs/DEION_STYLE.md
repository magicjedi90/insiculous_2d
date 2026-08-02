# DEION STYLE — The Insiculous World Bible & Asset Guide

**Status: DRAFT (F1) — castings and palette are PROPOSALS awaiting Jesse's
sign-off.** The `.sheet.ron` schema froze Jul 30 2026 (E3/E4 shipped) — sprite
frame art AND sheet layouts may both proceed; sidecars are stable to author
against (see `PROJECT_ROADMAP.md`, Phase E history).

---

## 1. World Bible

**The world is food-coded — and beverages are ordinary citizens.** Every
arena, obstacle, enemy, and prop reads as food, drink, or kitchen: countertop
battlegrounds, soup rivers, cake-brick fortresses, descending bread squadrons.
Liquid characters are NOT hero-exclusive: sodas, coffees, and other beverages
walk this world like anyone else. Deion and Cubert pop on screen because of
their hero framing and unmistakable silhouettes (round ball + icicle mohawk;
plain cube), not because they're the only liquids in town.

Tone: playful arcade menace. Food is charming AND out to get you. The
"insiculous" (insanely-ridiculous) chaos tiers push the food world from
appetizing (Normal) to feral (Insiculous).

**Art style principle (of record):** **Geometry Wars simplicity, food-themed.**
Every character is a readable geometric silhouette — a ball, a cube, a
crescent, a slice, a disc — with an expressive face. **NOT humanoid**: no
human proportions, no realistic figures; limbs, where they exist at all, are
comedic sticks (see `gunguy_ref.png`). If a character can't be read as "a
shape with a face," it's off-style. The `docs/concepts/` sheets define this
even where the designs are drafts — they're black-and-white only because
coloring used to happen in-engine; final art is simple AND colorful, in
Aseprite, against the palette in §4.

**Canonical view: side-on first.** *Deion the Insiculous* — the flagship
title this is all building toward — is a **side-scrolling platformer**, so
each character's iconic reference piece is drawn side-view. Per-game variants
use whatever view that game needs (top-down for Frogger/Snake, etc.) and
derive from the iconic side piece; the silhouette must survive the rotation.

**Puns are canon.** Names, flavor text, docs, and achievements lean into food
puns and dad jokes as house style — it keeps development entertaining
(precedent: Bananakin, Master Pi, In-Bread Yokels, DEIONized water).
Localization is part of the same pillar: the shipped **pirate-English locale**
(Pong, Frogger) is canon goofiness, and every game eventually gets the pirate
treatment.

**Legacy characters note:** the villain concepts in `docs/concepts/` predate
the food-theme decision and are due a **food-ification pass** — new
food-inspired designs and pun names, Jesse's call. Captain Michael's canadian
bacon grenade launcher is his existing food credential and survives any
redesign.

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
- **Chaos forms** (one drawn form per `ChaosMode` tier — Deion literally
  changes phase as the chaos heats up):
  | Mode | Form | Cell |
  |------|------|------|
  | Normal | Water ball (design above) | 16×16 |
  | Insane | **Steam** — wispy vapor ball, **mohawk turns to water** (liquid spikes/droplets; the whole guy heats up one phase) | 16×16 |
  | Ridiculous | **Solid ice** — frozen ball, ice mohawk (same read as Breakout's wrecking-ball frozen Deion, now the universal Ridiculous form) | 16×16 |
  | Insiculous | **Giant steam ball with a giant ICE mohawk** — steam + ice at once, both tiers embodied | **32×32** (2×2 cells — real footprint, honest collider) |

  Steam reads via dithered pale grays/whites from the cream/plate-neutrals
  ramp (export rules forbid semi-transparent pixels — no alpha wisps); the
  Insane mohawk uses the water ramp. **One sheet per form**
  (`deion_16` / `deion_steam_16` / `deion_ice_16` / `deion_insiculous_32`),
  each carrying the standard clip names — games pick the sheet by chaos mode
  at spawn and clip names stay the stable API. The 32px form needs its own
  sheet anyway (a sidecar declares exactly one cell size).

**Cubert** — Deion's best friend, an ice cube. The Diddy to his Kong, the
Luigi to his Mario. **Default P2 character across all 2-player modes.**
- Square silhouette (contrast to Deion's round one), same ice-family palette,
  pale-cyan faces with frost sparkle.
- Physical comedy is his thing: he slides, he slips, he falls over on
  collision. Signature clips: `slip` (feet-out pratfall) and `slide`.
- Projectile flavor when he shoots: ice chips/frost shards (smaller, scrappier
  than Deion's icicles).

**Dr. Maxwell** — the big bad, Deion's arch-rival and nemesis. A devil's food
cake with horns. Concept reference: `docs/concepts/dr_maxwell_ref.png`
(design already food-canon).
- Dark chocolate-cake body (near-black browns), frosting details, small
  glowing horns (candy-red ramp). Menacing but still a cake.
- Appears as boss/UFO/fortress-owner across games — the recurring villain.

**In-Bread Yokels** — Dr. Maxwell's rank-and-file: toast slices with a fried
egg in the middle. Crust-brown ramp + egg-white/yolk (cream + cheese ramps).
The egg is the face. Dopey, numerous, march in formation.

**The Flying Funk** — Deion's spaceship (easter egg: named after a song by
Jesse's friend). The name appears on the hull/HUD wherever the ship shows up.

### Dr. Maxwell's Rogues Gallery (concept drafts — food-ification pending)

All villains, all on Maxwell's side. **Every entry is a style draft**: the
silhouette and personality are the keepers; each is due a food-inspired
redesign plus (where not already a food pun) a pun rename by Jesse. The
gallery gives each game a candidate mid-boss/lieutenant; per-game castings
stay proposals in §5. In-Bread Yokels remain the rank-and-file.

**Captain Michael** — `docs/concepts/cpt_michael_ref.png`. Small round dark
character. Signature weapon: the **canadian bacon grenade launcher** — lobbed
bacon-disc grenades are his projectile language and his food credential.

**Bananakin** — `docs/concepts/bananakin_ref.png`. A banana with a face
(Star-Wars pun — the name already IS a food pun, likely survives the pass).
Concept is uncolored line art; color pass = cheese/butter yellows ramp.

**Master Pi** — `docs/concepts/master_pi_ref.png`. Round ninja with a
headband; pi/pie pun is already food-adjacent — lean the redesign toward
actual pie (crust body, lattice top).

**Mr. Crowley** — `docs/concepts/mr_crowley_ref.png`. Top hat + mustache,
occult-dandy vibe. Needs a food identity + pun rename.

**Gunguy** *(working name)* — `docs/concepts/gunguy_ref.png`. Stick-thin
soldier hauling a huge shoulder cannon. Needs a food identity + pun name;
role TBD (candidate: Maxwell's artillery man).

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

**Outline style — SETTLED (Jesse, Aug 2 2026): selective outline.** Outlines
are drawn in a **darker shade of the material's own ramp**, and only where the
silhouette needs separation from the background — not a universal black line
around everything. Reads more natural, stays simple, and matches actual SNES
practice. `#14101F` (near-black plum) stays in the palette as the deepest
dark, for the darkest materials and extreme-contrast edges — it is no longer
mandatory everywhere.

**Water & ice (open to any liquid/beverage character — heroes stay distinct
via silhouette, not color):**
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

Rule of thumb: **max 4 shades per material, selective outline drawn from the
ramp's own dark end**; a 16px sprite should rarely use more than 2 ramps.

**Character → ramp mapping:** Deion + Cubert = water/ice family (the ramp is
shared with other liquid characters now — the mohawk/cube silhouettes carry
the heroes). Deion steam forms = cream/plate neutrals (dithered wisps) + water
ramp for the Insane mohawk, rim-highlight ice for Insiculous. Dr. Maxwell =
devil's-food darks + candy-red horns/eyes. In-Bread Yokels = crust browns
(toast) + cream ramp (egg white) + cheese ramp (yolk). Flying Funk hull =
plate neutrals with an ice-family canopy. Rogues gallery (pre-food-ification
baselines): Bananakin = cheese/butter yellows; Master Pi = crust browns +
cream (pie) with a dark headband; Captain Michael + Mr. Crowley =
`#14101F`-anchored darks + plate neutrals with candy-red accents; bacon
grenades = candy/sauce reds + cream.

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
| **Breakout** | Paddle = **butter-pat on toast** (co-op: P2 = Cubert's frost-toast paddle); ball = Deion | **Dr. Maxwell's devil's-food fortress** — brick wall = cake segments with frosting mortar; armored bricks = foil-wrapped squares (foil tears per hit) | Falling pickups = **sprinkles / candy capsules**; Maxwell taunt cameo on level clear | Wrecking-ball mode = Deion frozen solid — same sprite family as the universal Ridiculous ice form (§1 chaos forms) |
| **Space Invaders** | P1 Deion firing **mohawk icicles**; P2 Cubert firing **ice chips** | Bunkers = **burger buns** (bites taken as they degrade) | Descending ranks of **In-Bread Yokels** (2–3 toast designs, egg faces); UFO flyby = **Dr. Maxwell** on a cake saucer | Yokels do a march-wiggle per step |
| **Snake** | **Cubert collecting ice cubes** — each cube joins the train behind him. Death = his signature **slip-and-fall** on collision | Kitchen-floor tile arena | Pellet = **ice cube** (from a leaky freezer); versus = the other trail | Versus proposal: P1 Cubert (cube trail) vs P2 Deion (water-droplet trail) — or two Cuberts, palette-shifted |
| **Asteroids** | Deion piloting the **Flying Funk** (name painted on the hull — easter egg); P2 co-op = Cubert's ship (name TBD by Jesse) | Asteroids = tumbling **popcorn chunks** (3 sizes) | UFO = **Dr. Maxwell's cake saucer** | Ship shots = icicle spikes. Drawn rotation set (16 angles) vs runtime rotation — decide at the Asteroids re-skin (Phase G) |
| **Frogger** | Deion **hopping home to the ice-cube tray** (home slots = tray sockets, filling with a frozen Deion each); P2 = Cubert | Road lanes = **conveyor belts of rolling food carts** (sushi rolls, hot dogs, donuts); river = **soup**; logs = **celery sticks / baguettes** | Crocs = **snapping hot-dog buns**; turtles = **crackers** that sink into the soup | Thematically the anchor game: water-guy crossing food traffic to reach the freezer |

Connective tissue across all games: Deion (or Cubert) as the player,
Dr. Maxwell's forces as the opposition, In-Bread Yokels as the rank-and-file,
icicle projectiles, and the Flying Funk wherever a ship is called for.

**Chaos-mode hero forms:** wherever Deion is the player, Insane / Ridiculous /
Insiculous swap in the corresponding form sheet from §1 (steam / ice / giant).
Insiculous is a **32×32 footprint** — games must account for the honest 2×2-cell
collider (physics ignores `Transform2D.scale`; there is no cheating with scale).

**Rogues gallery castings:** the §1 gallery villains are available as per-game
bosses/lieutenants — castings TBD after the food-ification pass; nothing is
hard-assigned yet.

## 6. File & Naming Conventions

```
docs/concepts/                    # reference/concept art (not shipped)
  deion_ref.png
  bananakin_ref.png  cpt_michael_ref.png  dr_maxwell_ref.png
  gunguy_ref.png     master_pi_ref.png    mr_crowley_ref.png
../games/deion_assets/            # canonical source (synced per game, F2)
  ai/                                 # AI-generated STAND-INS ONLY (see rule below)
  characters/deion/deion_16.png       + deion_16.sheet.ron
  characters/deion/deion_steam_16.png # chaos forms, one sheet per form
  characters/deion/deion_ice_16.png
  characters/deion/deion_insiculous_32.png
  characters/cubert/cubert_16.png
  characters/maxwell/maxwell_32.png
  characters/yokel/yokel_16.png
  characters/michael/michael_32.png   # rogues gallery (working names —
  characters/bananakin/bananakin_32.png   renamed when pun names land; cheap
  characters/master_pi/master_pi_32.png   now, nothing ships yet)
  characters/crowley/crowley_32.png
  characters/gunguy/gunguy_32.png
  ships/flying_funk_32.png
  tiles/<set>_16.png                  # gen_tiles output lands here too
  props/<prop>_16.png
```

This tree is **reserved naming**, not a mkdir order: directories are created
when the F2 sync task (or an actual asset drop) populates them. Only `ai/`
exists ahead of content.

- Files: `snake_case`, suffix `_16`/`_32` = cell size of the grid.
- **AI-asset quarantine rule (hard policy):** every AI-generated asset lives
  under `../games/deion_assets/ai/` AND carries an `ai_` filename prefix
  (e.g. `ai_bananakin_48.png`). AI art is **stand-in only** — it never ships
  to a digital storefront (AI-generated art is heavily frowned upon there;
  it's great for placeholders, not finished products). The folder + prefix
  make the pre-release purge greppable: `scripts/check_no_ai_assets.sh <dir>`
  must pass (zero `ai_*` files) on any shipping build. The suffix on AI files
  is the image's **actual** pixel size, never an aspiration.
- Every sheet PNG ships with a same-stem `.sheet.ron` sidecar (schema froze
  Jul 30 2026, `sheet_file.rs`; agents write sidecars, not Jesse).
- **Clips are the stable API**: game code references clip *names* only —
  `idle`, `walk`, `jump`, `hurt`, `attack`, `die` (lowercase snake_case).
  Grid positions may be rearranged freely as long as names survive.
- Placeholder sheets (F4) use final clip names + final cell sizes with
  blocked-out colors — swapping in real art must never touch game code.

## 7. Responsibility Split

**Jesse (hand-drawn, Aseprite):**
- Deion hero sheet: `idle`, `walk`, `jump`, `hurt` minimum (+ per-game
  variants: hopping) — building on `docs/concepts/deion_ref.png`.
- Deion **chaos-form sheets**: steam + ice at 16×16, insiculous at 32×32
  (§1 chaos forms table).
- Cubert sheet: `idle`, `walk`/`slide`, `slip` (the pratfall), `hurt`.
- Dr. Maxwell (at least a menacing `idle` + saucer/fortress appearances).
- In-Bread Yokel (1–3 toast variants, `march` clip).
- The Flying Funk (+ Cubert's ship if co-op ships differ; name Cubert's ship).
- The rogues-gallery **food-ification pass**: food-inspired redesigns + pun
  renames (Michael keeps the bacon launcher; Bananakin keeps the name; color
  pass on Bananakin; food identities for Crowley + Gunguy; real name for
  Gunguy) and gallery sign-off.
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

Chaos forms add **no new clip names**: each form is its own sheet carrying
this same vocabulary, and games swap sheets (not clips) per `ChaosMode`.
