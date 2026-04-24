# Lua-to-Rust Conversion Pipeline

The goal is to convert Mari0 into a Rust-backed IW2WTH codebase without losing the ability to run and compare the original behavior. This is a staged port, not a rewrite.

## Working Rules

- Keep the LÖVE/Lua game as the playable baseline until a Rust runtime can replace it.
- Move only deterministic, engine-neutral rules into Rust first.
- Do not move modules that call `love.*` until the needed rendering, audio, input, filesystem, or networking adapter exists.
- Every converted system needs a source Lua module, a Rust target module, and a verification gate.
- New IW2WTH gameplay should wait until the corresponding inherited Mari0 system is isolated or converted.

## Rust Workspace

- `rust/crates/iw2wth_core`: engine-neutral gameplay data and rules.
- `rust/xtask`: project tooling for inventory and conversion support.
- `docs/lua-inventory.generated.md`: generated Lua module inventory.
- `docs/conversion-ledger.md`: manual migration status and next targets.
- `docs/conversion-roadmap.md`: completion definition, phase plan, active next queue, and automation contract.

Useful commands:

```sh
cargo test --manifest-path rust/Cargo.toml
lua tools/generate_portalcoords_fixtures.lua > rust/crates/iw2wth_core/tests/fixtures/legacy_portalcoords.generated.tsv
cargo run --manifest-path rust/Cargo.toml -p xtask -- inventory --write docs/lua-inventory.generated.md
cargo run --manifest-path rust/Cargo.toml -p xtask -- validate-content --write docs/content-validation.generated.md
```

## Conversion Lanes

`core candidate`: Lua modules without direct `love.*` calls. These are first candidates for Rust because they can usually be tested without a renderer.

`adapter-bound`: Lua modules that mix gameplay with LÖVE calls. Split these into pure rules and runtime adapters before converting.

`runtime split`: oversized app/runtime files like `main.lua`, `menu.lua`, `game.lua`, and `editor.lua`. These should be decomposed before any direct Rust port.

## Stages

1. Baseline and inventory
   - Keep the current Lua game runnable.
   - Regenerate the inventory after large Lua changes.
   - Identify one small conversion target at a time.

2. Rust shadow core
   - Add Rust versions of pure data structures and rules.
   - Prefer small modules with unit tests.
   - Keep Lua as the source of truth until tests prove parity.

3. Golden behavior tests
   - Capture Lua behavior for movement, collision, portal/wormhole transit, and level parsing.
   - Encode those expectations as Rust tests.
   - Add edge cases before removing Lua behavior.

4. Adapter boundary
   - Define traits or narrow interfaces for input, rendering, audio, filesystem, and assets.
   - Keep engine choices out of `iw2wth_core`.
   - Decide later whether the Rust runtime is Bevy, ggez, macroquad, or a custom shell.

5. Runtime replacement
   - Build a minimal Rust playable slice.
   - Compare against the Lua baseline.
   - Move IW2WTH-specific gameplay into Rust only after the slice is stable.

## Initial Targets

1. Complete `variables.lua` constants into typed Rust config beyond the initial movement/gravity subset.
2. Expand `physics.lua` collision conversion from axis classification into full response fixtures.
3. Expand typed interpretation for lower-use linked, platform, portal, and level-control entities as runtime slices need them.
4. Expand lower-use linked, platform, portal, and level-control entity interpretation as runtime slices need them.

## Converted Seed Behavior

- `variables.lua` movement and gravity constants now exist in `iw2wth_core::config`.
- `physics.lua` `aabb` overlap behavior and `checkcollision` axis classification now exist in `iw2wth_core::collision`.
- `physics.lua` horizontal and vertical default collision snap/speed-zeroing response now exists in `iw2wth_core::collision`.
- `physics.lua` passive default collision snap/speed-zeroing response now exists in `iw2wth_core::collision`.
- Representative `physics.lua` `portalcoords` behavior now exists in `iw2wth_core::wormhole::legacy_portal_coords`, with a Lua-generated all-facing-pair fixture table consumed by Rust integration tests.
- `game.lua` `inrange` behavior now exists in `iw2wth_core::collision::in_range`.
- `mario.lua` normal horizontal movement and jump impulse now exist in `iw2wth_core::player`.
- `mario.lua` underwater horizontal movement and swim jump impulse now exist in `iw2wth_core::player`.
- `mario.lua` orange-gel ground probe and movement overrides now exist in `iw2wth_core::player`.
- `mario.lua` run and swim frame advancement now exists in `iw2wth_core::player`.
- `mario.lua` player gravity selection and `physics.lua` gravity velocity application now exist in `iw2wth_core::player`.
- `mario.lua` basic floor/head collision state transitions now exist in `iw2wth_core::player`.
- `mario.lua` invisible-tile player collision suppression now exists in `iw2wth_core::player`.
- `mario.lua` blue-gel floor and side bounce behavior now exists in `iw2wth_core::player`.
- `mario.lua` side-tile one-tile-gap run response now exists in `iw2wth_core::player`.
- `mario.lua` side-box collision response now exists in `iw2wth_core::player`.
- `mario.lua` side-button collision response now exists in `iw2wth_core::player`.
- `mario.lua` right-side pipe entry decision now exists in `iw2wth_core::player`.
- `mario.lua` non-invisible ceiling tile response now exists in `iw2wth_core::player`.
- `mario.lua` enemy stomp player bounce speed/state now exists in `iw2wth_core::player`.
- `goomba.lua` basic Goomba spawn, walking update, side collision turnaround, stomp expiry, shot motion, and portal rotation recovery now exist in `iw2wth_core::enemy`.
- `goomba.lua` walking spikey spawn, animation frames, Goomba-speed easing, and side-collision facing now exist in `iw2wth_core::enemy`.
- `goomba.lua` falling/Lakito-thrown spikey spawn state, animation frames, Lakito mask decay, floor landing transition, and nearest-player direction quirk now exist in `iw2wth_core::enemy`.
- `koopa.lua` basic Koopa spawn, walking/shell update, stomp shell transitions, shot motion, side collisions, start-fall, flying floor-bounce state, and red Koopa edge-turn behavior now exist in `iw2wth_core::enemy`.
- `koopa.lua` beetle walking/shell state and `fireball.lua` beetle fireball resistance now exist in `iw2wth_core::enemy`.
- `plant.lua` pipe plant spawn offsets, animation timing, movement cycle, player-near cycle hold, and shot removal now exist in `iw2wth_core::enemy`.
- `bulletbill.lua` projectile spawn offsets, lifetime, rotation recovery, direction selection, shot/stomp motion, offscreen removal, scissor-release state, and portal kill flag now exist in `iw2wth_core::enemy`.
- `bulletbill.lua` rocket launcher strict timer gate, viewport gate, nearest-player fire decision, max-count gate, and reset-to-next-time contract now exist in `iw2wth_core::enemy`.
- `mario.lua` spring and faithplate player state now exists in `iw2wth_core::player`.
- `mario.lua` fire/upfire/hammer player hazard collision outcomes now exist in `iw2wth_core::player`.
- `mappacks/*/settings.txt` parsing now exists in `iw2wth_core::content::MappackSettings`.
- Mari0 level grid and semicolon property parsing now exists in `iw2wth_core::content::Mari0Level`.
- High-use legacy entity cell IDs now have typed placement interpretation in `iw2wth_core::content`, preserving raw parameters and link targets.
- `xtask validate-content` validates all bundled mappack settings and level files with the Rust parser.
- Level cells now expose entity IDs and link targets, and `xtask validate-content` reports legacy entity usage across the bundled corpus.

The current player port intentionally excludes rendering quad selection and audio side effects.
The orange-gel path uses a narrow surface-query callback so runtime code can supply tile gel data
without moving map storage into the core crate.
