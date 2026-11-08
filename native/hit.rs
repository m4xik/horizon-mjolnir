// Runs before Horizon's hurt_enemy hook: a player's hit landed on a mob.
fn hit_strike(game: &mut crate::game::Game<'_>, stack: &ItemStack, entity: u32) {
    if &*stack.item != "mjolnir:mjolnir" {
        return;
    }
    let host: &mut dyn horizon_client::mod_api::Host = game;
    let Some(player) = host.acting_player() else { return };
    let Some(target) = host.entity(entity) else { return };
    if let Some(pos) = target.position(&*host) {
        let origin = [player.position[0], player.position[1] + 1.4, player.position[2]];
        crate::game::mjolnir::queue_hit(&player.dimension, pos, origin, &player.name);
    }
}
