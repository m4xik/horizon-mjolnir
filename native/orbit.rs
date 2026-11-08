// Client: while the cinematic runs the camera swings around the player and looks at them.
fn camera_orbit(game: &Game<'_>, position: &mut [f64; 3], yaw: &mut f32, pitch: &mut f32) {
    // Orbit the player's own eye (in third person `position` is already behind them).
    let eye = match game.native_acting_player() {
        Some(p) => [p.position[0], p.position[1] + 1.62, p.position[2]],
        None => *position,
    };
    if let Some((cam, y, p)) = crate::game::mjolnir::orbit(eye, *yaw) {
        *position = cam;
        *yaw = y;
        *pitch = p;
    }
}
