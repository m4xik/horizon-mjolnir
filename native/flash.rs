// Client: a white-blue flash over the whole picture, drawn after the GUI.
fn flash(game: &Game<'_>, batch: &mut HudBatch) {
    let argb = crate::game::mjolnir::flash_argb();
    if argb >> 24 == 0 {
        return;
    }
    // GUI pixels; the rectangle is larger than any window, so it covers the screen.
    game.native_fill(batch, [0.0, 0.0, 8192.0, 8192.0], argb);
}
