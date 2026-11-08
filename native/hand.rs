// Client: hide the first-person hand while the camera orbits, so the hammer is not drawn twice.
fn hide_hand(canceled: &mut bool) {
    if crate::game::mjolnir::cine_amount() > 0.05 {
        *canceled = true;
    }
}
