# I Want to Worm the Hole

Prototype started from the open source [Mari0](https://github.com/Stabyourself/mari0) engine.

## Run

Install LÖVE 11.4, then run:

```sh
love .
```

## Current Baseline

- Uses the upstream Mari0 LÖVE engine as the starting point.
- Renames the app/save identity to `iw2wth`.
- Sets the window and build product name to `I Want to Worm the Hole`.
- Disables upstream Mari0 online update and online mappack calls by default.
- Adds a Rust workspace under `rust/` for staged Lua-to-Rust conversion work.

## Rust Conversion

The Rust workspace is intentionally separate from the LÖVE runtime:

```sh
cargo test --manifest-path rust/Cargo.toml
cargo run --manifest-path rust/Cargo.toml -p xtask -- inventory --write docs/lua-inventory.generated.md
cargo run --manifest-path rust/Cargo.toml -p xtask -- validate-content --write docs/content-validation.generated.md
```

See `docs/conversion-pipeline.md` for the migration plan.

## Distribution Notes

The engine code is based on Mari0, whose upstream README states it is MIT licensed. This repository still contains upstream development assets and mappacks from Mari0; replace any assets you do not have rights to before public distribution.
