// Client: impact frames and the flash, plus cinematic letterbox bars, drawn after the GUI.
fn flash(game: &Game<'_>, batch: &mut HudBatch) {
    // The batch knows the window in GUI pixels, whatever the player's GUI scale.
    let (w, h) = (batch.width, batch.height);
    let argb = crate::game::mjolnir::flash_argb();
    if argb >> 24 != 0 {
        game.native_fill(batch, [0.0, 0.0, w, h], argb);
    }
    let amount = crate::game::mjolnir::cine_amount();
    if amount > 0.001 {
        let bar = h * 0.12 * amount;
        game.native_fill(batch, [0.0, 0.0, w, bar], 0xFF00_0000);
        game.native_fill(batch, [0.0, h - bar, w, bar], 0xFF00_0000);
    }
}
