// Runs at the end of every server tick: apply the actions that are due.
fn tick(game: &mut crate::game::Game<'_>, is_end: bool) {
    if !is_end {
        return;
    }
    for action in crate::game::mjolnir::drain() {
        match action {
            crate::game::mjolnir::Action::Command(line) => {
                game.native_run_command(&line);
            }
            crate::game::mjolnir::Action::Bolt { dim, pos, big } => {
                game.native_summon(
                    &dim,
                    "minecraft:lightning_bolt",
                    pos,
                    horizon_world::nbt::Compound::default(),
                );
                crate::game::mjolnir::trigger_fx(pos, big);
            }
        }
    }
}
