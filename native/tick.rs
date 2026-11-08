// Runs at the end of every server tick: apply the actions that are due.
fn tick(game: &mut crate::game::Game<'_>, phase: &horizon_client::mod_api::ServerTickPhase) {
    if !matches!(phase, horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. }) {
        return;
    }
    for action in crate::game::mjolnir::drain() {
        match action {
            crate::game::mjolnir::Action::Command(line) => {
                game.native_run_command(&line);
            }
            crate::game::mjolnir::Action::Bolt { dim, pos, big } => {
                crate::game::mjolnir::trace(&format!("summon bolt pos={pos:?} big={big}"));
                game.native_summon(
                    &dim,
                    "minecraft:lightning_bolt",
                    pos,
                    horizon_world::nbt::Compound::default(),
                );
                crate::game::mjolnir::trigger_fx(pos, big);
            }
            crate::game::mjolnir::Action::Arc { from, to } => {
                crate::game::mjolnir::add_arc(from, to);
            }
        }
    }
}
