// Client: blue-white light where the bolts landed. Horizon validates it (within 32 blocks of the player,
// loaded chunk, radius 0.25 to 16, intensity 0 to 1) and budgets it per mod.
fn glow(lights: &mut Vec<NativeColoredLight>) {
    for (at, intensity) in crate::game::mjolnir::lights_now() {
        lights.push(NativeColoredLight { at, rgb: 0x8FC8FF, radius: 16.0, intensity });
    }
}
