# IW2WTH Conversion Roadmap

This roadmap is the source of truth for the Mari0-to-Rust conversion. The ledger records what has moved; this file defines what remains, what order to use, and what "complete" means.

## Completion Definition

The conversion is complete when all of these are true:

- The IW2WTH game can run from a Rust runtime without requiring LÖVE/Lua for normal gameplay.
- Legacy Mari0 content needed by IW2WTH loads through Rust-owned parsers and typed data.
- Core gameplay rules used by IW2WTH are owned by Rust and covered by focused parity tests.
- Remaining Lua code, if any, is either archived as reference material or isolated behind a documented compatibility adapter.
- New IW2WTH-specific mechanics begin from Rust-owned systems rather than direct edits to legacy Mari0 gameplay code.
- The release/build path packages Rust runtime assets, third-party notices, and IW2WTH identity without upstream online-service dependencies.

## Current State

- Lua/LÖVE remains the playable baseline.
- Rust workspace exists under `rust/` with `iw2wth_core`, `iw2wth_runtime`, and `xtask`.
- Rust already owns seed math, level grid, content parsing, high-use, power-up, singleton marker, groundlight, gel-tile, linked/logic, platform, warp, lightbridge, laser, gel dispenser, faithplate, box/pushbutton, fire-hazard, goal/checkpoint, Bowser boss, bullet-bill launcher, Goomba-family, Koopa-family, Cheep-family, Hammer Bro, Lakito, squid, spring, and level-control marker entity placement typing, collision classification, portal transit parity with fixture coverage, content validation tooling, basic Goomba, walking/falling spikey, Koopa, beetle, plant, bullet bill, Cheep Cheep, squid, Lakito, thrown hammer projectile, Hammer Bro, and flying fish enemy state machines including red Koopa edge-turns and launcher fire decisions, mushroom, one-up, star, and flower item state, spring object state, held-spring player update, vine object state, `grabvine` player transition, on-vine movement timer/frame update, on-vine tile/portalwall collision clamps, on-vine down-portal probe output, on-vine `checkportalHOR` success/bounce outputs, player fireball projectile state/collision outcomes, fire/upfire/castle fire hazard state, firework boom, coin-block animation, block debris, and scrolling-score effect state, coin-block reward, breakable-block side-effect, item-on-top block-jump request, enemy-on-top block-shot request, coin-on-top collection, block-bounce scheduling, bounce-completion item replay/removal, bounce-queue prune/sprite-batch regeneration, many-coins timer countdown, many-coins timer lookup, contained reward-block reveal, empty breakable-block destroy gate, and block-hit sound contracts, a growing subset of player movement/state rules, and runtime audio/rendering intent plus tile metadata query boundaries, including structured harness/CLI frame audio command detail for direct coin-pickup, player fireball launch, fireball collision release/callback summaries, fireball collision score-counter, scrolling-score animation, enemy-hit intent source provenance, automatic fireball projectile/enemy overlap probes from adapter snapshots, projected fireball enemy hit/removal snapshots, projected fireball projectile collision snapshots, projected active-fireball count snapshots, player render-intent atlas quad/source-rectangle and color-layer image previews, block-hit, contained reward-reveal, coin-block reward, top-coin collection, block-break, portal open/fizzle, and portal-enter transit stop/play sequencing.
- The conversion is not complete because the Rust runtime shell can load and step one parsed level with one-player movement/collision intent, invisible-tile collision suppression, a first small-player coin pickup interaction with structured harness/CLI detail and frame audio command detail, report-only player fireball launch intents plus projectile update/removal progression, map/tile target probes, collision outcome probes, automatic projectile/enemy overlap probes, collision release/callback summaries, fireball collision score-counter source provenance, explicit and overlap-derived enemy-hit intent summaries and projected enemy hit/removal snapshots from adapter snapshots, projected fireball projectile collision state threaded into later report-only projectile update/progression previews, and projected active-fireball count snapshots with structured harness/CLI detail, deterministic seeded projectile reproduction, and source-specific frame audio command detail, a first non-invisible ceiling block-hit intent, report-only block-bounce scheduling, report-only block-bounce timer progression/prune updates, report-only block-bounce item-spawn intents, report-only contained reward-block reveal/audio intents, structured harness/CLI block-bounce schedule/progress/item-spawn and contained reward reveal details, report-only single coin-block reward intents, report-only many-coins reward intents, structured harness/CLI single/many coin-block reward and top-coin collection details, report-only many-coins timer countdown/start/projection intents, report-only top-coin collection intents, report-only reward tile-change projection intents, adapter-side projected reward tile-change snapshots for future shell map queries, harness frame/final counts for projected reward tile-change snapshots, report-only coin-count/life-reward counter intents, report-only score-counter and immediate/deferred scrolling-score intents, report-only coin-block animation progression intents, report-only scrolling-score animation progression intents, report-only block-debris animation progression intents, structured harness/CLI effect animation progress/prune details, report-only item-on-top jump request intents, report-only enemy-on-top shot request intents, report-only empty breakable-block destroy intents, report-only breakable-block cleanup projections, structured harness/CLI reward tile-change projection and breakable-block cleanup details, report-only current-level portalability counts, report-only portal target probes, report-only portal open/fizzle outcome intents, structured harness/CLI target placement and open/fizzle outcome details, structured harness/CLI reservation projection and replacement details, adapter-side projected portal state snapshots, harness frame/final counts for projected portal state snapshots, deterministic CLI seed flags for projected portal slots, forced player body/speed, seeded fireball projectiles, collision probes, and fireball enemy snapshots, report-only jump items, top enemies, and many-coins timers, structured harness/CLI item jump request, enemy shot request, many-coins timer progress/start, coin counter, life reward counter, score counter, scrolling-score details, player tile-collision frame details, and player render-intent atlas quad/source-rectangle and color-layer image details, report-only portal-pair readiness summaries, report-only portal-transit candidate probes, report-only portalcoords preview reports with adapter-side blocked-exit probes, report-only portal-transit outcome summaries, structured harness/CLI readiness/candidate/outcome details, report-only portal-enter audio intent reports for both successful and blocked-exit transit outcomes, adapter-side projected player-state snapshots from portal-transit outcomes threaded into later portal-transit and portal target probes, structured harness/CLI source-selection details for projected-player-source portal target probes, structured harness/CLI detail for projected portal-state block-hit guards, and source-specific structured harness/CLI frame audio details for block-hit, contained reward-reveal, coin-block/top-coin coin, block-break, portal open/fizzle, and portal-enter transit command sequencing, but it is not yet an interactive rendered playable loop, golden fixture coverage is still limited to portal transit, on-vine player frame steps, default collision responses, and runtime harness composition, and large gameplay/runtime systems still live in Lua; input, parsed-level map-query, deterministic frame-clock, filesystem/assets, audio intent, rendering intent, legacy tile metadata decoding, one-level shell, adapter-side one-player frame integration, legacy player-spawn discovery, and a runnable local harness boundary now exist in `iw2wth_runtime`.

## Operating Rules

- Keep the Lua baseline runnable until the Rust runtime has a comparable playable slice.
- Convert deterministic, engine-neutral behavior first.
- Add a test gate for every converted rule before using it as live behavior.
- Keep adapter-bound behavior out of `iw2wth_core`; represent it with narrow data/context structs or callbacks.
- Update `docs/conversion-ledger.md` whenever ownership changes.
- Update this roadmap when phase status or next priorities change.

## Roadmap

### Phase 0: Baseline and Attribution

Status: Seeded

- Preserve Mari0 source as the upstream reference.
- Rebrand IW2WTH identity, title/icon assets, save identity, and packaging metadata.
- Keep third-party attribution for Mari0.
- Disable inherited online mappack/update calls unless explicitly reintroduced through IW2WTH-owned services.

Exit criteria:

- `git diff --check` passes.
- Lua baseline remains available for behavior comparison.

### Phase 1: Inventory and Validation Tooling

Status: Seeded

- Maintain generated Lua inventory at `docs/lua-inventory.generated.md`.
- Maintain bundled content validation at `docs/content-validation.generated.md`.
- Parse mappack settings, level grids, cell values, links, and entity usage through Rust tooling.

Exit criteria:

- `cargo run --manifest-path rust/Cargo.toml -p xtask -- inventory --write docs/lua-inventory.generated.md` succeeds after major Lua structure changes.
- `cargo run --manifest-path rust/Cargo.toml -p xtask -- validate-content --write docs/content-validation.generated.md` succeeds with 0 errors.

### Phase 2: Engine-Neutral Rust Core

Status: In progress

Completed slices:

- Math primitives and level grid.
- Core movement/gravity constants.
- Underwater, orange-gel, and animation constants.
- AABB overlap, `inrange`, and collision-axis classification.
- Horizontal/vertical and passive default collision response snap and speed-zeroing contracts.
- Representative legacy portal transit behavior.
- Legacy portal transit fixture coverage for all facing pairs.
- High-use legacy entity placement typing from bundled content usage.
- Power-up block reward entity typing for mushroom, one-up, star, and many-coins markers.
- Player spawn, plant, drain, button, and maze-gate singleton marker typing, including button links and maze gate number parameters.
- Ground light orientation entity typing with link preservation.
- Gel tile surface entity typing with gel ID parameter preservation.
- Basic Goomba spawn, walking update, side turnarounds, stomp, and shot state.
- Walking spikey spawn, animation frames, Goomba-speed easing, and side-collision facing.
- Falling/Lakito-thrown spikey spawn, animation frames, Lakito collision mask decay, floor landing transition, and preserved nearest-player selection quirk.
- Basic Koopa spawn, walking/shell update, side collision, stomp, shot, flying floor-bounce state, and red Koopa edge-turn map query.
- Beetle Koopa walking/shell state and fireball resistance.
- Plant spawn, animation timing, pipe movement cycle, player-near cycle hold, and shot removal state.
- Bullet bill projectile spawn, lifetime, rotation recovery, direction selection, shot/stomp motion, offscreen removal, scissor-release state, portal kill flag, and launcher timer/fire decision.
- Cheep Cheep spawn, color speed selection, vertical movement, animation, shot motion, rotation recovery, and collision default suppression.
- Squid spawn, idle/upward/downward swim state, nearest-player trigger/direction choice, shot motion, rotation recovery, and collision default suppression.
- Lakito spawn, strict throw/respawn timers, spikey-count gate, projected-player tracking with preserved nearest-player quirk, passive Lakito-end transition, shot/stomp state, and spikeyfall collision default suppression.
- Thrown hammer projectile spawn, direction speed, gravity constants, strict animation timer, mask-slot decay, portal kill flag, and collision default suppression.
- Hammer Bro enemy spawn, active patrol bounds, strict throw timer, injected jump choice, jump-mask clearing, nearest-player turning, animation/prepare frames, preserved negative speed easing quirk, shot/stomp motion, side/ceiling/floor kill interactions, portal jump-mask clearing, emancipation kill, and preserved Lua nil-direction shot quirk.
- Flying fish spawn, injected spawn speed randomness, zero-speed correction, animation direction, strict animation timing with the uninitialized-frame quirk, shot/stomp motion, rotation recovery without modulo, and collision default suppression.
- Player fireball projectile spawn, strict star-animation timing, explosion animation/removal, offscreen release gates including the legacy inactive-X quirk, side/floor/ceiling/passive collision default contracts, block-hit sound events, enemy shot events, score outputs, and Bowser/beetle fireball quirks.
- Fire hazard static and Bowser spawn offsets, Bowser target selection through injected random drop, strict two-frame animation timing, leftward motion, vertical target seeking with clamp, and collision default suppression.
- Upfire hazard spawn, hidden baseline position, manual gravity/motion, strict delay relaunch timing with injected next delay, zero-delay launch guard, and collision default suppression.
- Castle fire hazard spawn defaults, explicit length/direction handling, strict rotation timer, clockwise/counter-clockwise angle wrapping, strict four-frame animation timing, and child fire segment position/frame derivation.
- Firework boom effect spawn random offsets through injected values, immediate score delta, sound threshold crossing, strict removal timing, and strict three-frame selection.
- Coin-block animation effect spawn position, strict frame-delay loop, inclusive frame-31 removal threshold, and legacy floating-score output without score total mutation.
- Block debris effect spawn position/speed, strict two-frame animation toggling, gravity-before-motion update order, and strict below-screen removal threshold.
- Scrolling-score effect spawn label/position with scroll-relative X capture, strict lifetime removal, and numeric-score versus 1up presentation offset derivation.
- Coin-block reward side-effect outputs from `mario.lua`, including used-block tile mapping, coin/score/animation outcomes, exact 100-coin life reward behavior, and many-coins timer behavior.
- Breakable-block side-effect outputs from `mario.lua`, including portal-protection no-op behavior, clear-tile/clear-gels outputs, block-break sound, 50-point score event, four debris spawns, and sprite-batch regeneration.
- Item-on-top jump request outputs from `mario.lua` `hitblock`, including `jumpitems` filtering, inclusive horizontal center range checks, exact tile-top bottom alignment, and source-block X propagation for mushroom/one-up jump handlers.
- Enemy-on-top shot request outputs from `mario.lua` `hitblock`, including inclusive horizontal center range checks, exact tile-top bottom alignment, left-versus-right shot direction selection from the block center, 100-point score event placement, and preserved duplicate probe entries when the legacy enemy list repeats a bucket.
- Coin-on-top collection outputs from `mario.lua` `hitblock`, including the optional top-coin guard, top coin tile clear to ID 1, coin sound, 200-point score event, exact 100-coin life reward/reset behavior, and the top-coin-position coin-block animation spawn.
- Block-bounce scheduling outputs from `mario.lua` `hitblock`, including the epsilon timer start, block coordinate capture, contained mushroom/one-up/star/vine replay payloads, manycoins/empty false-content scheduling, hitter-size propagation, and immediate sprite-batch regeneration at bounce start.
- Bounce-completion item replay/removal outputs from `game.lua` blockbounce updates, including the strict `>` completion threshold, timer clamp to `blockbouncetime`, empty-entry removal without replay, mushroom-to-flower upgrade for big hitters, vine-at-block replay coordinates, and the preserved exact-threshold no-delete quirk.
- Bounce-queue prune and post-completion sprite-batch regeneration outputs from `game.lua` blockbounce updates, including descending batch removal of completed entries and regeneration when any bounce entry is pruned.
- Many-coins timer countdown updates from `game.lua`, including positive-only countdown, preserved negative overshoot after a decrement crosses zero, and no-op behavior once the timer is non-positive.
- Many-coins timer lookup outputs from `mario.lua` `hitblock`, including block-coordinate matching and preserved last-match duplicate semantics when repeated timer entries target the same block.
- Contained reward-block reveal outputs from `mario.lua` `hitblock`, including used-block tile mapping for spriteset/invisible combinations and the vine-versus-mushroom appearance sound selection for non-manycoins contents.
- Empty breakable-block destroy gate outputs from `mario.lua` `hitblock`, including missing/small hitter rejection, coin-block and non-empty-content suppression, and delegation into breakable-block portal protection versus destruction side effects.
- Block-hit sound output from `mario.lua` `hitblock`, including suppression behind the portal guard, editor mode, and out-of-map early returns before the unconditional `blockhitsound` call.
- Mushroom item spawn/body flags, emergence timing with the legacy previous-timer threshold, portal rotation recovery, side/floor/ceiling collision collection outcomes, side bounce directions, and block-jump impulse/direction.
- One-up item spawn/body flags with the legacy inactive start, emergence timing, portal rotation recovery, post-emergence offscreen/destroy removal, side/floor/ceiling collision reward outcomes, side bounce directions, and block-jump impulse/direction.
- Star item spawn/body flags, gravity, emergence timing and half-jump launch, strict four-frame animation timing, portal rotation recovery, side/floor/ceiling collision star-power outcomes, floor bounce/default suppression, and block-jump impulse/direction.
- Flower item spawn/body flags, gravity-free emergence with the legacy strict greater-than snap, strict four-frame animation timing, player growth collection outcomes, side default suppression, floor/ceiling active gates, and no-op block-jump behavior.
- Spring object spawn/body flags, hit-timer reset, fully-extended no-op behavior, strict timer clamp, and the legacy compression frame sequence from `spring.lua`.
- Held-spring player update branch from `mario.lua`, including X pinning, timer advance without clamp, player Y placement from `springytable`, and the strict `>` auto-release threshold before leave-spring.
- Vine object spawn/body flags from `vine.lua`, including start-variant limit selection, upward growth timing, limit clamp behavior, and the non-negative growth height floor.
- `mario.lua` `grabvine` transition contract, including the ducking reset before the portal guard, portal suppression, climb-state setup, vine anchor capture, and side-dependent X snap plus pointing angle.
- On-vine movement update branch from `mario.lua`, including movement timer/frame advancement for up/down climbing, idle timer reset, direction-specific move speeds/frame delays, and the `<= vineanimationstart` trigger gate.
- On-vine movement update branch from `mario.lua`, including up/down tile and portalwall collision clamps, neutral no-op collision behavior, and the down-climb `checkportalHOR` probe Y emitted before solid collision clamping.
- Lua-generated on-vine player frame-step fixture coverage for up, down, idle, and blocked down-vine motion cases.
- Lua-generated horizontal default collision response fixture coverage for left/right movement, Lua `false` handler suppression, and static targets.
- Lua-generated vertical default collision response fixture coverage for up/down movement, Lua `false` handler suppression, and static targets.
- Lua-generated passive default collision response fixture coverage for passive handler dispatch, default floor snapping, upward-speed preservation, and Lua `false` floor handler suppression.
- On-vine `checkportalHOR` result branch from `mario.lua`/`physics.lua`, including horizontal portal entry selection, speed-facing gates, successful teleport output, blocked-exit vertical bounce, non-opposite-pair jump/fall flags, and exit-facing portaled callback output.
- `mario.lua` `dropvine` transition contract, including the side-dependent `7/16` X offset, falling animation-state swap, gravity restore, and vine mask clear outputs.
- On-vine attachment-loss probe from `mario.lua`, including the empty vine-overlap gate that triggers `dropvine` only after the vine overlap check returns empty.
- Player horizontal movement, jump impulse, underwater movement, orange-gel surface overrides, animation frame advancement, gravity stepping, basic floor/head collision state, blue-gel bounce behavior, enemy stomp bounce speed/state, spring/faithplate player state, and fire/upfire/hammer hazard collision outcomes.
- Mario side-tile one-tile-gap run response.
- Mario invisible-tile player collision suppression.
- Mario side-button collision response.
- Mario side-box collision response.
- Mario right-side pipe entry decision.
- Mario non-invisible ceiling tile response.
- Door, emancipation grill, wall indicator, timer, and not-gate entity typing with preserved links/parameters.
- Oscillating, falling, spawner, seesaw, and bonus platform entity typing with preserved width/type parameters.
- Pipe, pipe-spawn, warp-pipe, and vine entity typing with preserved destination parameters.
- Directional lightbridge entity typing.
- Laser emitter and detector entity typing with preserved detector links.
- Blue, orange, and white gel dispenser entity typing.
- Directional faithplate entity typing.
- Companion cube, box tube, and pushbutton entity typing with preserved links.
- Castle fire, fire-start, and upfire hazard entity typing with preserved castle fire length parameters.
- Flag, axe, and checkpoint goal/progression entity typing.
- Bowser boss entity placement typing.
- Bullet-bill launcher marker entity typing.
- Goomba-half, spikey, and spikey-half enemy placement typing.
- Koopa-half, red Koopa, red Koopa-half, beetle, beetle-half, red flying Koopa, and flying Koopa enemy placement typing.
- Hammer Bro enemy placement typing.
- Lakito enemy placement typing.
- Red and white Cheep Cheep enemy placement typing.
- Squid enemy placement typing.
- Spring player-interaction placement typing.
- Maze, bullet-bill zone, Lakito-end, and flying-fish zone level-control marker typing.
- Report-only player fireball collision release/callback summaries, including `mario:fireballcallback` count-delta metadata and no live projectile-queue or fireball-counter mutation.
- Report-only player fireball release summaries projected into adapter-side active-fireball count snapshots for later fire-guard probes, including launch and collision-release count deltas without live player-counter mutation.
- Report-only player fireball update/removal release/callback summaries, including offscreen `released_thrower` `mario:fireballcallback` count-delta metadata and projected active-fireball count snapshots without live player-counter mutation.
- Report-only player fireball map/tile target probes from adapter-side projectile positions over Rust-decoded tile metadata and projected reward tile changes, including tile-hit/block-hit metadata and no live projectile collision mutation.
- Report-only player fireball map/tile target probes threaded into cloned tile collision outcome and release summaries, including side-tile block-hit sound/release metadata, floor-bounce no-release behavior, projected fireball-count deltas, and no live projectile collision mutation.
- Structured fireball collision probe source provenance for explicit probe requests versus automatic map-derived tile collisions, including release summaries and projected fireball-count labels without live projectile/counter mutation.
- Fireball collision source provenance threaded into report-only score-counter summaries for explicit enemy probes, including CLI source labels and no live score, enemy, block, audio, or projectile mutation.
- Fireball collision score source provenance threaded into report-only scrolling-score animation summaries for explicit enemy probes, including shell queue/progress reports, harness frame/final detail counts, CLI labels, and no live rendering or enemy mutation.
- Explicit fireball enemy collision reports threaded into adapter-side enemy-hit intent summaries from provided enemy snapshots, including `b:shotted("right")` direction, optional score metadata, harness/CLI detail, and no live enemy mutation.
- Explicit fireball enemy-hit intents threaded into adapter-side projected enemy hit/removal snapshots, including Lua `shotted` active/shot state, later report-only enemy-query suppression for the same projected enemy, harness/CLI detail, and no live enemy mutation.
- Adapter-side projected fireball enemy hit/removal state threaded into later explicit fireball enemy score and scrolling-score queries, suppressing repeated report-only score effects for removed enemies while preserving first-hit Lua score metadata and no live enemy mutation.
- Adapter-side fireball enemy snapshots threaded into automatic report-only projectile/enemy overlap probes, including projected enemy hit/removal suppression, overlap-derived enemy-hit intents, score provenance, harness/CLI detail counts, and no live enemy or projectile mutation.
- Lua-baseline fixture coverage for fireball projectile/enemy overlap ordering across representative targets, including passive, horizontal, vertical, missing-handler skip, Bowser no-score, and beetle no-shot/no-score cases consumed by `iw2wth_runtime::shell` tests.
- Adapter-side projected fireball projectile collision snapshots from report-only collision probes, including first-probe-per-projectile ordering, later automatic collision-query suppression, harness/CLI detail exposure, and no live projectile queue mutation.
- Adapter-side projected fireball projectile collision state threaded into report-only projectile update/progression previews, including explosion animation/removal timing, harness progress detail exposure, and no live projectile queue mutation.
- Report-only fireball projectile render-intent previews from live/projected projectile states, including flying versus explosion frame metadata, image/quad/draw coordinates, harness/CLI detail exposure, and projected removal suppression without live rendering or projectile queue mutation.
- Report-only player render-intent previews from the one-player shell state, including legacy body, facing, render-frame selection, size/power-up, ducking, fire-animation metadata, harness/CLI detail exposure, focused frame-selection matrix tests, and no live rendering or player mutation.
- Report-only player render-intent quad/source-rectangle previews from the one-player shell state, including legacy small/big/fire atlas frame coordinates, harness/CLI detail exposure, focused atlas-coordinate tests, and no live rendering or player mutation.
- Report-only player render-intent color-layer image previews from the one-player shell state, including legacy `graphic[1]`, `graphic[2]`, `graphic[3]`, then `graphic[0]` draw order, small/big animation image paths, player/flower/white tint metadata, harness/CLI detail exposure, and no live rendering or player mutation.

Next queue:

1. Thread report-only player render-intent hat draw previews into `iw2wth_runtime::shell` and harness/CLI summaries, preserving the legacy post-color-layer, pre-`graphic[0]` hat draw order without live rendering or player mutation.

Exit criteria:

- `cargo test --manifest-path rust/Cargo.toml` passes.
- `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` passes.
- Converted rules have unit tests or generated fixture tests that explain the Lua parity being preserved.

### Phase 3: Golden Fixture Harness

Status: Complete for initial coverage

- Add a repeatable way to capture Lua behavior for small deterministic scenarios.
- Prefer fixture generation for portal transit, player movement frame steps, collision responses, and entity parsing.
- Store fixture data in a stable format that Rust tests can consume.
- Portal transit now has a checked-in Lua-generated fixture table consumed by Rust integration tests.
- On-vine player frame steps now have a checked-in Lua-generated fixture table consumed by Rust integration tests.
- Horizontal default collision responses now have a checked-in Lua-generated fixture table consumed by Rust integration tests.
- Vertical default collision responses now have a checked-in Lua-generated fixture table consumed by Rust integration tests.
- Passive default collision responses now have a checked-in Lua-generated fixture table consumed by Rust integration tests.

Exit criteria:

- At least portal transit, player frame steps, and collision responses have golden cases generated from Lua or manually justified where Lua execution is unavailable.
- Rust tests fail clearly when parity drifts.

### Phase 4: Adapter Boundaries

Status: In progress

- Define Rust traits or data boundaries for input, rendering, audio, filesystem/assets, map queries, and time.
- Keep the adapter layer separate from `iw2wth_core`.
- Decide the runtime shell only after the core contracts are stable enough to support a playable slice.

Completed slices:

- `iw2wth_runtime::input` defines the first adapter-side legacy input polling boundary outside `iw2wth_core`, including keyboard, joystick hat/button/axis bindings, strict joystick deadzone behavior, and projection into `PlayerMovementInput`.
- `iw2wth_runtime::map` defines an adapter-side parsed-level map query boundary outside `iw2wth_core`, including Lua-style 1-based tile lookups, map bounds, and top-gel payload projection for core surface movement callbacks.
- `iw2wth_runtime::time` defines an adapter-side deterministic frame-clock boundary outside `iw2wth_core`, including legacy raw-`dt` clamping, speed-target easing/snap, scaled `gdt`/update `dt`, and `frameadvance`/`skipupdate` gates.
- `iw2wth_runtime::assets` defines an adapter-side filesystem/assets boundary outside `iw2wth_core`, including legacy mappack settings/level path construction, custom tiles availability, custom music `.ogg`/`.mp3` precedence, and custom background fallback order.
- `iw2wth_runtime::audio` defines an adapter-side audio intent boundary outside `iw2wth_core`, including legacy sound-effect IDs, Lua global names, asset paths, portal sound volumes, score-ring looping metadata, and `playsound` sound-enabled stop/play command sequencing.
- `iw2wth_runtime::render` defines an adapter-side rendering intent boundary outside `iw2wth_core`, including legacy background colors, nearest-clamped tile atlas image specs, SMB/Portal/custom tile-batch atlas ordering, fractional-scroll tile-batch offsets, 17-pixel tile atlas stride counts, and custom-background back-to-front parallax tiling.
- `iw2wth_runtime::tiles` defines an adapter-side legacy tile metadata boundary outside `iw2wth_core`, including the `quad.lua` six-row alpha flag layout, PNG byte loading, and SMB/Portal/custom tile ID ordering.
- `iw2wth_runtime::map` now exposes an adapter-side tile metadata map query that joins Lua-style parsed level coordinates to Rust-decoded `quad.lua` collision, invisible, breakable, coin-block, coin, and portalability flags without moving atlas/image policy into `iw2wth_core`.

Exit criteria:

- A minimal Rust runtime can call core update functions through adapter interfaces without direct Lua data structures.
- Rendering/audio/input decisions do not leak into engine-neutral core modules.

### Phase 5: Minimal Rust Playable Slice

Status: In progress

- Build a small Rust runtime scene with player movement, map collision, and content loading.
- Compare against equivalent Lua behavior.
- Keep scope narrow: one level, one player, core movement/collision, no broad feature rewrite.

Completed slices:

- `iw2wth_runtime::shell` loads one Mari0-derived level through the filesystem/assets adapter, exposes a Lua-style parsed-level map query, steps the deterministic frame clock, projects legacy input, and emits rendering/audio intents without moving adapter-bound behavior into `iw2wth_core`.
- `iw2wth_runtime::shell` can step a narrow one-player movement/gravity/tile-collision frame over the parsed-level map query, using existing core movement/collision rules and an adapter-side tile-collision callback rather than moving tile atlas policy into `iw2wth_core`.
- `iw2wth_runtime::harness` exposes a local runnable deterministic command around the one-level shell/player frame integration, using filesystem assets, fixed keyboard input, adapter-side tile solidity, and explicit known-parity-gap reporting without expanding gameplay scope.
- `iw2wth_runtime::harness` discovers the one-player start from parsed legacy player-spawn entities using Lua row-major scan order and the player-1 `startx - 6/16`, `starty - 1` placement formula, while preserving the fixed-seed fallback when no spawn marker exists.
- `iw2wth_runtime::harness` feeds the shell tile-collision callback from Rust-decoded legacy tile atlas metadata, preserving the adapter boundary while replacing the temporary `tile_id != 1` solidity stub.
- `iw2wth_runtime::shell` uses the adapter-side tile metadata map query for the first narrow non-collision map interaction: small-player coin pickup probes from `mario.lua`, including coin tile clear to ID 1, coin sound intent, 200-point pickup report, and no repeat pickup after the map tile is cleared.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for direct player coin-pickup frame reports, preserving pickup tile, tile-clear, score, and sound metadata without executing live counters or rendering.
- `iw2wth_runtime::shell` threads adapter-side invisible tile metadata into the one-player collision path, suppressing floor and side default collision resolution with the existing core Lua-parity helpers while keeping tile atlas policy outside `iw2wth_core`.
- `iw2wth_runtime::shell` threads adapter-side tile metadata into the first non-invisible ceiling block-hit intent, including hit tile coordinates, breakable/coin-block flags, head-bump state, block-hit sound intent, and invisible-ceiling suppression without starting broader block-bounce behavior.
- `iw2wth_runtime::shell` threads ceiling block-hit intents into report-only block-bounce schedules using parsed legacy cell contents and the Rust-owned core scheduling contract, preserving the Lua epsilon timer, block coordinate, small-player hitter size, mushroom/one-up/star/vine replay mapping, many-coins replay suppression, and no map mutation or reward replay.
- `iw2wth_runtime::shell` threads report-only block-bounce schedules into an adapter-side timer progression/prune report, preserving the Lua update-before-player ordering, strict completion threshold, prune-triggered sprite-batch regeneration flag, suppressed replay-spawn payload, and no map mutation or live reward replay.
- `iw2wth_runtime::shell` threads completed block-bounce replay payloads into explicit adapter-side item-spawn intent reports, including source queue index/coordinate, mushroom-to-flower upgrade for big hitters, harness intent counts, and no map mutation or live item object execution.
- `iw2wth_runtime::shell` threads block-hit/block-bounce scheduling reports into report-only contained reward-block reveal and audio intents, preserving the Lua used-block tile mapping, spriteset selection, vine-versus-mushroom appearance sound choice, many-coins suppression, harness intent counts, and no map mutation or live item object execution.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only block-bounce schedule/progress/item-spawn and contained reward reveal reports, preserving queue coordinates, timers, hitter size, replay spawn metadata, reveal tile/sound metadata, and no live item execution.
- `iw2wth_runtime::shell` threads plain coin-block hit reports into report-only single coin-block reward intents, preserving coin sound, 200-point score, coin-block animation position, used-block tile-change mapping, harness intent counts, and no map/counter/life mutation.
- `iw2wth_runtime::shell` threads many-coins block-hit reports into report-only reward intents, preserving coin sound, 200-point score, coin-block animation position, missing-timer spawn intent, expired-timer used-block tile-change intent, harness counts, and no map/counter/life/live-timer mutation.
- `iw2wth_runtime::shell` threads block-hit reports into report-only top-coin collection intents, preserving the top-coin tile-clear intent, coin sound, 200-point score, coin-block animation position, exact 100-coin life reward report, harness counts, and no map/counter/life/top-coin mutation.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose structured detail for report-only single/many coin-block reward and top-coin collection reports, preserving block/coin coordinates, coin sounds, animation state, score/coin/life outcomes, tile-change metadata, many-coins timer spawns, and no live counter/map mutation.
- `iw2wth_runtime::shell` threads block-hit reports into report-only item-on-top jump request intents from adapter-provided item snapshots, preserving mushroom/one-up selection, inclusive center-range checks, exact tile-top bottom alignment, missing jump-handler suppression, source-block X propagation, harness counts, and no live item mutation.
- `iw2wth_runtime::shell` threads block-hit reports into report-only enemy-on-top shot request intents from adapter-provided enemy snapshots, preserving inclusive center-range checks, exact tile-top bottom alignment, left/right shot direction from the block center, 100-point score placement, missing shotted-handler suppression, duplicate probe preservation, harness counts, and no live enemy mutation.
- `iw2wth_runtime::shell` threads block-hit reports into report-only empty breakable-block destroy intents for eligible big-player hits, preserving portal-protection no-op reporting, portal-guarded suppression of block-hit/block-break sounds and bounce/reward side effects, clear-tile/clear-gel/tile-object-removal intents, block-break sound, 50-point score, debris intents, sprite-batch regeneration, harness counts, and no map/gels/collision-object/live-debris mutation.
- `iw2wth_runtime::shell` threads small-player coin pickups and single/many/top coin reward reports into report-only coin-count and life-reward counter intents from adapter-side shell snapshots, preserving exact 100-coin reset and disabled-life one-up sound semantics, harness counts, and no live counter mutation.
- `iw2wth_runtime::shell` threads small-player coin pickups, single/many/top coin reward reports, enemy-on-top shot reports, and empty breakable-block destroy reports into report-only score-counter intents from adapter-side shell snapshots, preserving Lua `addpoints` ordering, immediate scrolling-score spawn intents only for coordinate-bearing enemy shot scores, harness counts, and no live score mutation.
- `iw2wth_runtime::shell` threads report-only coin-block animation progression into deferred scrolling-score intents, preserving `coinblockanimation.lua` frame-31 `addpoints(-200, x, y)` behavior as a 200-point floating score with no live score mutation.
- `iw2wth_runtime::harness` exposes report-only coin-block animation progress/prune counts in frame and aggregate summaries plus CLI output without executing live rendering.
- `iw2wth_runtime::shell` threads report-only scrolling-score animation progression into frame reports for scrolling scores spawned by score-counter and coin-block animation reports, preserving `scrollingscore.lua` timer/removal and numeric-versus-1up presentation offsets without live rendering.
- `iw2wth_runtime::harness` includes nonzero local scenario coverage for report-only block-debris and scrolling-score animation progress/prune counts without turning effect animations into live rendering.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only coin-block animation, block-debris animation, and scrolling-score animation progress/prune reports, preserving queue index, effect state, remove/prune flags, floating-score/presentation metadata, and no live rendering.
- `iw2wth_runtime::shell` threads report-only many-coins timer countdown progression into frame reports and same-frame many-coins block-hit lookup, preserving `game.lua` positive-only countdown, negative overshoot, update-before-player ordering, harness counts, and no live timer mutation.
- `iw2wth_runtime::shell` threads report-only many-coins timer start/projection handling into frame reports, preserving `mario.lua` missing-timer insertion after a many-coins hit, after-countdown timer-table projection, harness start counts, and no live timer mutation.
- `iw2wth_runtime::shell` threads report-only reward tile-change projections into frame reports for contained reward reveals, coin-block rewards, top-coin collections, and empty breakable-block destroys, preserving Lua map-write intent ordering, harness counts, and no live map mutation.
- `iw2wth_runtime::shell` stores report-only reward tile-change projections in an adapter-side projected tile-change snapshot and feeds future shell map metadata queries from that overlay, preserving ordered Lua map-write intent history while leaving the parsed level unchanged.
- `iw2wth_runtime::harness` exposes adapter-side projected reward tile-change snapshot counts in final and frame summaries and carries the ordered projection history in the final frame report without mutating the parsed level.
- `iw2wth_runtime::shell` threads report-only breakable-block cleanup projections into frame reports for empty breakable-block destroys, preserving `destroyblock` tile-collision object removal, gel clearing, four debris spawns, sprite-batch regeneration ordering, harness counts, and no live object/gels/debris/sprite-batch mutation.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only reward tile-change projections and breakable-block cleanup projections, preserving ordered source/tile/action/debris metadata and no live map/object/gels/debris mutation.
- `iw2wth_runtime::shell` threads report-only block-debris animation progression into frame reports for debris spawned by breakable-block cleanup projections, preserving `blockdebris.lua` following-frame update timing, gravity-before-motion movement, frame toggles, removal pruning, harness counts, and no live rendering/debris mutation.
- `iw2wth_runtime::harness` exposes report-only block-debris and scrolling-score animation progress/prune counts in frame and aggregate summaries without executing live rendering.
- `iw2wth_runtime::harness` exposes report-only current-level portalability query counts from adapter-owned tile metadata, including queried, portalable, solid-portalable, and solid-non-portalable totals without adding live portal placement or portal physics.
- `iw2wth_runtime::shell` threads adapter-side player aim/input snapshots into report-only portal target probes over current-level portalable-solid metadata, preserving portal slot requests, trace hits, placement feasibility, harness counts, CLI flags, and no live portal placement or physics.
- `iw2wth_runtime::shell` threads requested portal target probes into report-only portal open/fizzle outcome intents, preserving slot-specific open sounds, rejected-target fizzle sound, harness and CLI counts, and no live portal placement or portal physics.
- `iw2wth_runtime::shell` threads portal open outcomes into report-only portal tile/wall reservation projection intents, preserving Lua `modifyportaltiles` two-tile aperture coordinates, side-specific `portalwall:new` rectangles, harness/CLI counts, and no live portal placement or physics.
- `iw2wth_runtime::shell` threads report-only portal reservation projections into an adapter-side projected portal state snapshot, preserving per-slot replacement, derived future block-guard reservations, harness exposure, and no live portal physics.
- `iw2wth_runtime::harness` exposes adapter-side projected portal state snapshot counts in frame and aggregate summaries plus CLI output, preserving the per-slot projected state and keeping live portal physics out of scope.
- `iw2wth_runtime::shell` threads adapter-side projected portal state snapshots into the report-only block-hit portal guard context, preserving explicit adapter-supplied reservations, suppressing guarded block-hit side effects, and keeping live portal physics out of scope.
- `iw2wth_runtime::harness` exposes structured projected portal-state block-hit guard details in the harness report and CLI output, preserving projected-versus-explicit guard counts, protected tile coordinates, guarded tile ID/breakable/coin-block metadata, reservation facing metadata, and report-only suppression of hit sounds, block bounces, and live block effects.
- `iw2wth_runtime::shell` threads adapter-side projected portal state snapshots into report-only portal replacement summaries, preserving same-slot previous/replacement state, preserved peer-slot state, same-placement no-op suppression, harness/CLI counts, and no live portal physics.
- `iw2wth_runtime::harness` exposes deterministic CLI seed flags for initial projected portal slots and forced player body/speed, preserving Lua portal reservation shapes and allowing local projected portal-state block-hit guard reproduction without test-only setup.
- `iw2wth_runtime::harness` exposes deterministic CLI seed flags for report-only jump item, top enemy, and many-coins timer snapshots, preserving adapter-provided Lua-side object/timer state for local block-hit side-effect reproduction without test-only setup.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only item jump request, enemy shot request, and many-coins timer progress/start summaries, preserving block tile, item/enemy index, direction/score metadata, timer before/after/start metadata, and no live object, enemy, timer, or counter mutation.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only coin counter, life reward counter, score counter, and scrolling-score summaries, preserving source metadata, before/after counter values, life-reward payloads, scrolling-score labels/coordinates, and no live counter mutation.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only player tile-collision frame reports, preserving last horizontal collision, last vertical collision, and last ceiling block-hit tile metadata with no live rendering or broader gameplay execution.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only frame audio commands, preserving the direct player coin-pickup `coinsound` stop/play ordering before live audio execution.
- `iw2wth_runtime::harness` exposes source-specific structured harness/CLI detail for report-only frame audio commands from ceiling block hits and contained reward reveals, preserving `blockhitsound`, `mushroomappearsound`, and `vinesound` stop/play sequencing without live audio execution.
- `iw2wth_runtime::harness` exposes source-specific structured harness/CLI detail for report-only frame audio commands from coin-block rewards, top-coin collections, empty breakable block breaks, and portal open/fizzle outcomes, preserving `coinsound`, `blockbreaksound`, `portal1opensound`, `portal2opensound`, and `portalfizzlesound` stop/play sequencing without live audio execution.
- `iw2wth_runtime::harness` exposes source-specific structured harness/CLI detail for report-only frame audio commands from portal transit outcomes, preserving `portalentersound` stop/play sequencing for both successful and blocked-exit portal pass reports without live audio execution.
- `iw2wth_runtime::shell` threads report-only player fireball launch requests into adapter-side launch intents, preserving `mario.lua` flower-power, ducking, controls-enabled, and `maxfireballs` guards plus the Rust-owned fireball spawn offsets/direction and `fireballsound` command sequencing without live projectile execution.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail and deterministic `--fire`/`--fire-flower`/`--fireball-count` flags for report-only player fireball launch intents, preserving launch source/spawn/count/audio metadata with no live projectile object mutation.
- `iw2wth_runtime::shell` threads report-only player fireball projectile update/removal progression through an adapter-side projectile queue, preserving Rust-owned fireball animation timers, offscreen release/removal gates, and launch-to-progress handoff without executing live projectile collisions.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose deterministic `--seed-fireball` input plus structured projectile progress/prune counts and last-update details for report-only player fireball projectile progression, preserving frame, timer, release, and queue metadata with no live collision execution.
- `iw2wth_runtime::shell` threads report-only player fireball update/removal `released_thrower` outcomes into adapter-side release/callback summaries and projected active-fireball count snapshots, preserving offscreen `mario:fireballcallback` count deltas without mutating live player counters.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose structured fireball update/removal release summary details and projected active-fireball count labels, preserving projectile-update source metadata and no-live-counter-mutation flags for CLI reporting.
- `iw2wth_runtime::shell` threads adapter-side fireball projectile positions into report-only map/tile target probes over Rust-decoded tile metadata and projected reward tile changes, preserving Lua non-player invisible-tile skipping, target tile coordinates/IDs, side/floor/ceiling/passive axis classification, tile-hit block-hit-sound metadata, unchanged parsed level state, and no live projectile collision mutation.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose structured fireball map/tile target probe counts and last-probe details, preserving projectile source state, target tile flags, block-hit-sound metadata, and no-live-collision flags for CLI reporting.
- `iw2wth_runtime::shell` threads report-only fireball map/tile target probes into cloned tile collision outcome and release summaries, preserving side/ceiling/passive tile block-hit sound and `mario:fireballcallback` release metadata versus floor-tile bounce/no-release behavior, projected active-fireball count deltas, and no live projectile collision mutation.
- `iw2wth_runtime::harness` exposes map-derived fireball tile collision outcome/release counts and details through the existing report-only collision summaries, preserving block-hit audio intent, projected fireball-count snapshots, and no live block/projectile/audio/counter mutation.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose structured fireball collision source provenance that distinguishes explicit `--probe-fireball-collision` requests from automatic map-derived tile collision summaries, preserving release/projected-count labels and no live projectile/counter mutation.
- `iw2wth_runtime::shell` threads report-only player fireball collision probes through cloned projectile state, preserving Rust-owned side/floor/ceiling/passive collision outcomes for tile, portalwall, spring, bullet-bill, and enemy targets without mutating live enemies, blocks, audio, counters, or projectile physics.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose deterministic `--probe-fireball-collision` input plus structured collision probe counts and last-probe details, preserving before/after projectile state, default-suppression, released-thrower, block-hit-sound, enemy-shot, and score metadata with no live collision execution.
- `iw2wth_runtime::shell` threads report-only player fireball collision release/callback outcomes into adapter-side projectile release summaries, preserving `mario:fireballcallback` count-delta metadata without mutating live projectile queues or fireball counters.
- `iw2wth_runtime::shell` threads projected active-fireball count snapshots from launch and collision-release deltas into later fire guards, preserving Lua `maxfireballs` behavior while keeping live player counters unchanged.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose structured fireball collision release summary counts, last-release details, and projected active-fireball count details, preserving callback labels, launch/release count deltas, and no-live-mutation flags for CLI reporting.
- `iw2wth_runtime::shell` threads report-only player fireball collision probe outcomes into frame audio and score summaries, preserving tile-like `blockhitsound` stop/play command sequencing and enemy `addpoints(firepoints[a], self.x, self.y)` metadata without mutating live enemies, blocks, audio, counters, or projectile physics.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose source-specific fireball collision frame audio details and fireball-collision score source labels, preserving block-hit sound commands, score before/after summaries, scrolling-score coordinates, and no live audio or counter execution.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose explicit fireball enemy-collision score provenance through report-only scrolling-score animation summaries, preserving queued/progressed score source labels, source-specific harness frame/final counts, CLI detail output, and no live rendering or enemy mutation.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose explicit fireball enemy-hit intent summaries from adapter-provided enemy snapshots, preserving Lua `b:shotted("right")` direction, optional firepoints score metadata, deterministic `--seed-fireball-enemy` input, harness frame/final counts, CLI detail output, and no live enemy mutation.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose adapter-side projected fireball enemy hit/removal snapshots from explicit and automatic overlap-derived enemy-hit intents, preserving Lua `shotted` active/shot state, later report-only enemy-query suppression, harness frame/final counts, CLI detail output, and no live enemy mutation.
- `iw2wth_runtime::shell` gates later explicit fireball enemy score and scrolling-score reports through adapter-side projected enemy hit/removal state, preserving the first-hit Lua score metadata while suppressing repeated report-only score effects for removed enemies.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose automatic report-only fireball projectile/enemy overlap probes from adapter-provided enemy snapshots, preserving Lua passive overlap classification, `b:shotted("right")` metadata, firepoints score provenance, projected removal suppression, CLI source labels, and no live projectile or enemy mutation.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose adapter-side projected fireball projectile collision snapshots from cloned collision probes, preserving first-probe-per-projectile ordering and later automatic collision-query suppression without mutating the live projectile queue.
- `iw2wth_runtime::shell` threads report-only player fireball launch and collision-release summaries into an adapter-side projected active-fireball count snapshot, preserving `mario:fireballcallback` count deltas for later fire-guard probes without mutating live player counters.
- `iw2wth_runtime::harness` and `iw2wth-runtime-harness` expose projected active-fireball count snapshot counts and source/detail labels, preserving launch and collision-release before/delta/after metadata for CLI reporting without live player-counter mutation.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose report-only fireball projectile render-intent previews from live and projected projectile states, preserving flying/explosion frame metadata, image/quad/draw details, projected removal suppression, and no live rendering or projectile queue mutation.
- `iw2wth_runtime::shell`, `iw2wth_runtime::harness`, and `iw2wth-runtime-harness` expose report-only player render-intent previews from the one-player shell state, preserving body, facing, render-frame selection, size/power-up, ducking, fire-animation metadata, atlas quad/source-rectangle metadata, color-layer image draw order, structured CLI detail, and no live rendering or player mutation.
- `iw2wth_runtime::shell` threads adapter-side projected portal state snapshots into report-only portal-pair readiness summaries, preserving one-slot not-ready state, both-slot entry/exit pairing metadata, harness/CLI counts, and no live portal physics or transit.
- `iw2wth_runtime::shell` threads report-only portal-pair readiness summaries into portal-transit candidate probes for the player body center, preserving Lua `inportal` center tile math, entry/exit slot metadata, harness/CLI counts, and no live portal physics or player mutation.
- `iw2wth_runtime::shell` threads report-only portal-transit candidate probes into portalcoords preview reports with adapter-side blocked-exit probes, preserving entry/exit facing, player body, speed, rotation, animation-direction output metadata, blocking tile coordinates, and Lua blocked-exit bounce metadata with no live teleport/player mutation.
- `iw2wth_runtime::shell` threads portalcoords preview reports into report-only portal-transit outcome summaries, preserving successful teleport versus blocked-exit bounce preview classification, output body/speed metadata, harness/CLI counts, and no live teleport/player mutation.
- `iw2wth_runtime::harness` exposes structured harness/CLI detail for report-only portal-pair readiness, portal-transit candidate, and portal-transit outcome summaries, preserving last-frame tile/slot/facing/body/speed metadata and no live portal physics or player mutation.
- `iw2wth_runtime::shell` threads report-only portal-transit outcome summaries into adapter-side projected player-state snapshots, preserving teleport output body/speed/animation direction, blocked-exit bounce speed with the original body, harness/CLI counts, and no live player mutation.
- `iw2wth_runtime::shell` threads adapter-side projected player-state snapshots into subsequent portal-transit candidate probes and portalcoords previews, preserving projected center-tile/body/speed reads while keeping the live player body unchanged.
- `iw2wth_runtime::shell` threads adapter-side projected player-state snapshots into subsequent portal target probes, preserving projected portal ray source coordinates while keeping live aiming input and player body mutation out of scope.
- `iw2wth_runtime::harness` exposes projected-player-source portal target probe coverage in frame/final reports and CLI counts, preserving projected ray source coordinates while leaving the live player body unchanged.
- `iw2wth_runtime::harness` exposes structured report-only portal target source-selection details in the harness report and CLI output, preserving live-versus-projected source counts, final source coordinates, requested slot, aim angle, and no live portal/player mutation.
- `iw2wth_runtime::harness` exposes structured report-only portal target placement and open/fizzle outcome details in the harness report and CLI output, preserving possible-versus-impossible placement counts, final trace hit and placement coordinates, requested slot, outcome kind, sound intent, and no live portal/player mutation.
- `iw2wth_runtime::harness` exposes structured report-only portal reservation projection and replacement details in the harness report and CLI output, preserving Lua tile/wall reservation coordinates, previous/replacement/preserved slot summaries, and no live portal placement or physics.
- `iw2wth_runtime::shell` threads report-only portal-transit success and blocked-exit bounce outcomes into portal-enter audio intent reports, preserving the `physics.lua` `passed and j == "player"` sound trigger, harness/CLI counts, and no live audio execution or player mutation.
- `iw2wth_runtime::harness` exposes structured report-only portalcoords preview, portal-enter audio, and projected player-state snapshot details in the harness report and CLI output, preserving last-frame slot/facing/body/speed/rotation/sound/source metadata and no live portal physics, audio execution, or player mutation.

Known parity gaps:

- The Phase 5 shell is still not an interactive rendered playable loop.
- Rust-owned tile metadata is currently connected to harness collision, adapter-side map/tile queries, small-player coin pickup with structured harness/CLI direct-pickup and frame audio command details, source-specific player fireball launch, fireball map/tile target probes over parsed and projected tile state, map-derived fireball tile collision outcome/release summaries, source-specific fireball collision block-hit audio and score summaries, explicit and automatic overlap-derived fireball enemy-hit intent summaries, projected enemy hit/removal snapshots, projected fireball projectile collision snapshots, projected active-fireball count snapshots, report-only fireball projectile update/removal progression, report-only fireball projectile render-intent previews, report-only player render-intent previews with atlas quad/source-rectangle and color-layer image details, collision outcome probes, and collision release summaries, block-hit, contained reward-reveal, coin-block/top-coin coin, block-break, portal open/fizzle, and portal-enter frame audio details, invisible-tile collision suppression, the first ceiling block-hit intent, report-only block-bounce scheduling, report-only block-bounce timer progression/pruning, report-only block-bounce item-spawn intents, report-only contained reward-block reveal/audio intents with structured harness/CLI detail summaries, report-only single/many-coins coin-block reward intents and top-coin collection intents with structured harness/CLI reward detail summaries, report-only reward tile-change projection intents and future map-query snapshots, structured harness/CLI reward tile-change projection and breakable-block cleanup detail summaries, harness projected reward tile-change snapshot summaries, report-only coin-count/life-reward counter intents, report-only score-counter and immediate/deferred scrolling-score intents with structured harness/CLI detail summaries, report-only coin-block animation progression intents, report-only scrolling-score animation progression intents, report-only block-debris animation progression intents, structured harness/CLI effect animation progress/prune details, report-only many-coins timer countdown/start/projection intents, report-only item-on-top jump request intents, report-only enemy-on-top shot request intents, structured harness/CLI item jump, enemy shot, fireball enemy-hit, and many-coins timer detail summaries, structured harness/CLI player tile-collision detail summaries, report-only empty breakable-block destroy intents, report-only breakable-block cleanup projections, report-only current-level portalability counts, report-only portal target probes, report-only portal open/fizzle outcome intents, structured harness/CLI target placement and open/fizzle outcome details, structured harness/CLI portal reservation projection and replacement details, deterministic CLI seed flags for projected portal/player, jump item, top enemy, fireball enemy, many-coins timer, fireball launch snapshots, seeded fireball projectiles, fireball map target probes, fireball collision probes, and fireball release summaries, adapter-side projected portal state snapshots and harness counts, report-only portal-pair readiness summaries, report-only portal-transit candidate probes, report-only portalcoords previews with blocked-exit probes, report-only portal-transit outcome summaries with structured harness/CLI readiness/candidate/outcome details, report-only portal-enter audio intent reports with structured harness/CLI details, adapter-side projected player-state snapshots from portal-transit outcomes threaded into later transit and target probes with structured harness/CLI source-selection and snapshot details, and structured harness/CLI projected portal-state block-hit guard details; live reward objects, live projectile collision mutation, live enemy mutation, live portal placement/physics, and live coin/score/life side effects remain outside the shell slice.
- Object/entity, portal, gel physics beyond top-gel movement boosts, and rendering/audio execution remain outside the shell slice.

Exit criteria:

- A user can run the Rust slice locally.
- The slice loads a Mari0-derived level and supports basic movement/collision through Rust systems.
- Known parity gaps are documented in this roadmap.

### Phase 6: Gameplay System Migration

Status: Not started

- Migrate entities and systems by usage/risk order: common map entities, player interactions, simple enemies, portals/gels, then specialized level mechanics.
- Maintain the Lua baseline until each migrated subsystem has tests and a runtime adapter path.
- Retire Lua ownership only after the Rust runtime handles the corresponding gameplay path.

Exit criteria:

- Core Mari0-derived gameplay required for IW2WTH runs from Rust.
- Remaining gaps are intentionally cut, redesigned, or documented as non-goals.

### Phase 7: IW2WTH-Specific Game Development

Status: Blocked by earlier phases

- Start new wormhole/worm gameplay once inherited systems are isolated enough to avoid compounding legacy debt.
- Build new mechanics in Rust-owned modules by default.
- Keep compatibility with selected Mari0 content only where it supports IW2WTH design.

Exit criteria:

- IW2WTH-specific mechanics and content no longer depend on direct edits to Mari0 Lua gameplay code.

## Automation Contract

The `conversion-check` automation should use this file every run.

On each run:

1. Read this roadmap and `docs/conversion-ledger.md`.
2. Decide whether the completion definition is satisfied.
3. If incomplete, pick the first safe item from the roadmap's active next queue.
4. Implement one narrow slice.
5. Update the ledger and this roadmap if status or next priorities changed.
6. Run the relevant verification gates.
7. Open an inbox item summarizing completion status, the slice attempted, files changed, verification results, and blockers.

If a run cannot safely proceed, it should open an inbox item with the blocker and leave the automation active.
