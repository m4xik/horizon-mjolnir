// Runs before Horizon's use_item hook: a right click with any item.
fn wrath(game: &mut crate::game::Game<'_>, is_mjolnir: bool, claimed: &mut bool) {
    if !is_mjolnir {
        return;
    }
    let host: &mut dyn horizon_client::mod_api::Host = game;
    let Some(p) = host.acting_player() else { return };
    // Claiming the use makes the hand swing; the cooldown only gates the storm.
    crate::game::mjolnir::queue_wrath(p.uuid, &p.dimension, &p.name, p.position, p.yaw);
    *claimed = true;
}
