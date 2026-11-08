# Handoff: Mjolnir (Thor's hammer) for Horizon

Repository: https://github.com/m4xik/horizon-mjolnir (branch `main`). Local folder: `C:\Users\Administrator\Desktop\mjolnor\thor-mjolnir`.
Horizon is the Rust rewrite of Minecraft Java (`horizon-rust/horizon`). The mod is pinned to Horizon nightly
`fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` (2026-11-08) in `native/horizon-mixin.toml`.

## Layout
- `content/` - content mod (`.hmod` folder): item, recipe, creative tab, texture, lang, sounds. Copy it to
  `%APPDATA%\horizon\horizon-mods\mjolnir\` (already done once; re-copy after changes).
- `native/` - native mod (horizon-mixins bundle): `horizon-mixin.toml`, `runtime.rs` (pure logic, unit tests inside),
  helpers `hit.rs use.rs tick.rs shake.rs flash.rs glow.rs fov.rs orbit.rs hand.rs arcs.rs shader.rs view.rs`.
- `shader/mjolnir_storm/` - Horizon shader pack (impact frames, zoom blur, colour split, storm grade). Copy to
  `%APPDATA%\horizon\shaderpacks\` and pick it in Options > Video Settings > Shaders (already copied once).
- `licenses/` - Orbital Laser (MIT) licence and notes for the sounds used.

## What it does
- Hit a mob with the hammer: lightning bolt, sparks, arc from the hammer, crack sound.
- Right click (5 s cooldown): "Thor's wrath": storm, bolts marching forward, letterbox bars, FOV push-in, camera orbit
  with the player drawn (third person while it runs), impact frames, flash, blue light, shake.

## Status (last commit `298e207`)
Worked in game: bolts, shake, flash, bars, FOV push, hammer item (after fixes), recipe, hand-in-world.
Not yet checked in game after the last build: camera orbit with the player visible, lightning arcs, the shader pack,
the Orbital Laser sounds, no stray lightning when entering a world.
Hooks wired by raw `target` (not in the hook catalog at the pin, so likeliest to break a build):
`native_world_primitives` (arcs.rs) and `native_client_tick_post` (view.rs).

## Rules learnt the hard way
- The launcher installs a native mod as a snapshot of one commit. After every push: Remove the mod in the launcher,
  Add from GitHub with the repo link, Build ("Build anyway" if asked). The game must be closed while building.
- horizon-mixins bindings may only be target parameters or `&`/`&mut`/`&*self` of them. No expressions. Helper function
  name must equal the injection id with `-` as `_` and must not equal any target parameter name.
- Content mod loader accepts only `.json .png .ogg .toml` under `assets/` and `data/`; a stray `.txt` rejects the whole mod.
  Item JSON: `max_stack_size` and `rarity` are top-level keys; there is no `minecraft:unbreakable` component in 26.3
  (unbreakable = remove `max_damage` and `damage` with `"!minecraft:max_damage": {}`).
- Server and client halves share statics in `runtime.rs` (single player and LAN host only; a remote dedicated server needs mod messages).
- Logs: game `%APPDATA%\horizon\logs\latest.log`; build `%APPDATA%\horizon-launcher\native\build.log`;
  screenshots `%APPDATA%\horizon\screenshots`. Read them directly instead of asking the user.
- Docs used: `docs/mods.md`, `docs/native-mods.md`, `docs/shaders.md`, `docs/api/shaders-reference.md` in horizon-rust/horizon;
  hook catalog `native-hooks.json` (432 hooks at the pin). Orbital Laser notes: github.com/Stink-O/orbital-laser `AGENTS.md`.
- No Rust toolchain on the dev machine: the launcher compiles the mod. Unit tests in `runtime.rs` have never been run.
- Git: the first push may need `git -c credential.helper= push` (Git Credential Manager errored); later pushes worked with `git push origin main`.

## Next ideas
Check everything above in game; tune the orbit (radius 5.5, sweep 140 degrees), bar height (12%), flash times;
mod messages for dedicated servers; a stronger lightning shader glow.
