# Mjolnir - Thor's Hammer (Horizon mod)

Two parts:

| Part | Folder | What it gives |
|---|---|---|
| Content mod (`.hmod`) | `content/` | The item (unbreakable, 13 damage, slow swing, enchant glint), crafting recipe, creative tab, texture, RU/EN names |
| Native mod (`.hnmod`) | `native/` | Lightning on every hit, and "Thor's wrath" on right click |

Recipe: netherite ingots + lightning rod + sticks:

```
N L N
N S N
  S
```

## Effects
- **Hit a mob**: a lightning bolt, electric sparks, an explosion puff, thunder.
- **Right click** (5 second cooldown): the sky turns to a storm, the caster is ringed in sparks and struck harmlessly
  (Resistance, Fire Resistance, Speed, Jump Boost), then four bolts march forward and a big final bolt ends it.

## Install the content part (works without Rust)
Copy the `content` folder into Horizon's `horizon-mods/` folder (rename it `mjolnir`), enable it in Options > Mods.
Or install the tool with `cargo binstall --git https://github.com/horizon-rust/horizon cargo-horizon-mod`, then in
`content/` run `cargo horizon-mod build` and `cargo horizon-mod check target/horizon-mod/mjolnir.hmod`.

## Build the native part
Native mods are compiled into the game. Follow https://github.com/horizon-rust/horizon-native-mod-template :
copy `native/` over `mixins/example_mod/` (rename the module as the template README describes), then
`python3 tools/horizon.py stage --horizon ../horizon`, `check`, `test`, `build`. Linux and Rust 1.95 are required.
`horizon_revision` is pinned to the commit the template was validated on.

## Status
Not compiled or run: this machine has no Rust toolchain. Hook names, signatures and helpers were checked against the
Horizon sources, but expect small compile fixes in `native/*.rs` (for example the exact `Host::entity(..).position(..)`
call and whether `native_run_command` works in your game mode).
