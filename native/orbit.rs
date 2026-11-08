// Client: while the cinematic runs the camera swings around the player and looks at them.
fn camera_orbit(position: &mut [f64; 3], yaw: &mut f32, pitch: &mut f32) {
    if let Some((cam, y, p)) = crate::game::mjolnir::orbit(*position, *yaw) {
        *position = cam;
        *yaw = y;
        *pitch = p;
    }
}
