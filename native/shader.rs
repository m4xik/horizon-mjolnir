// Client: values for the mjolnir_storm shader pack, read there as frame.mods[0].
fn storm_values(params: &mut [[f32; 4]; 8]) {
    params[0] = crate::game::mjolnir::shader_values();
}
