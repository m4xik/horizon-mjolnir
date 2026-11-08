// Client: a slow push-in on the field of view while Thor's wrath plays.
fn fov_push(modifier: &mut f32) {
    *modifier *= crate::game::mjolnir::fov_scale();
}
