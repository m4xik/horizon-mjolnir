// Client: switch the camera to third person while the cinematic runs, so the player is drawn, and back after.
fn camera_view(game: &mut crate::game::Game<'_>) {
    use crate::game::perspective::CameraType;
    let current = if game.camera_type == CameraType::FirstPerson {
        0
    } else if game.camera_type == CameraType::ThirdPersonBack {
        1
    } else {
        2
    };
    if let Some(code) = crate::game::mjolnir::take_view_change(current) {
        game.camera_type = match code {
            1 => CameraType::ThirdPersonBack,
            2 => CameraType::ThirdPersonFront,
            _ => CameraType::FirstPerson,
        };
    }
}
