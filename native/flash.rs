// Client: impact frames and the flash, plus cinematic letterbox bars, drawn after the GUI.
fn flash(game: &Game<'_>, batch: &mut HudBatch) {
    let argb = crate::game::mjolnir::flash_argb();
    if argb >> 24 != 0 {
        // GUI pixels; the rectangle is larger than any window, so it covers the screen.
        game.native_fill(batch, [0.0, 0.0, 8192.0, 8192.0], argb);
    }
    let amount = crate::game::mjolnir::cine_amount();
    if amount > 0.001 {
        if let Some([w, h]) = game.native_physical_window_size() {
            // Minecraft's automatic GUI scale: the largest whole scale that keeps 320 x 240 GUI pixels.
            let scale = ((w as f32 / 320.0).min(h as f32 / 240.0).floor()).max(1.0);
            let (gw, gh) = (w as f32 / scale, h as f32 / scale);
            let bar = gh * 0.12 * amount;
            game.native_fill(batch, [0.0, 0.0, gw, bar], 0xFF00_0000);
            game.native_fill(batch, [0.0, gh - bar, gw, bar], 0xFF00_0000);
        }
    }
}
