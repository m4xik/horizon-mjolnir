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

## Screen effects (simple version)
Each bolt that lands also triggers, on the client: a white-blue **flash** over the screen (under half a second),
**camera shake** that decays over about a second (scaled by Distortion Effects in Options), and a **blue light** at
the strike point. The final bolts of the storm are twice as strong as the others.
These use a static shared between the server and client halves, so they work in single player and for a LAN host.
A remote dedicated server would need a mod message to the client (not done yet).

## Cinematic (right click)
Black letterbox bars slide in, the view pushes in slowly (field of view -16%), and the biggest bolts open with
three hard cuts - white, black, white, about a fifth of a second - before the white-blue flash fades.
The bars use the real window size in GUI pixels, so any GUI scale works.

## Camera, arcs, shader and sound
- **Camera orbit:** during the right-click cinematic the camera swings around the player, looking at them; the
  first-person hand is hidden meanwhile.
- **Lightning arcs:** a jagged blue-white arc (glow, core and two branches, flickering every 50 ms) from the hammer to
  every target and strike point (`native_world_primitives`).
- **Shader pack `mjolnir_storm`** (folder `shader/mjolnir_storm`, copy to `shaderpacks/` and pick it in Options >
  Video Settings > Shaders): the big strike's impact frames in the style of Orbital Laser's Kanada mode (white with
  black silhouettes and converging lines, a negative, white again), then a zoom blur with a colour split and glow, plus a
  cold storm grade and a vignette. Without the pack, the mod's HUD flash is used instead.
- **Sound:** layered vanilla sounds (charge-up, thunder, trident thunder, sonic boom, mace smash). Orbital Laser's
  synthesised sounds (MIT) can be added as a resource pack.

Idea credits: Orbital Laser by Stink-O (MIT), https://github.com/Stink-O/orbital-laser

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
