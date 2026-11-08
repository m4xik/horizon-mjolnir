// Client: adds the camera shake of the bolts that just landed.
fn shake(shake: &mut NativeShake) {
    let s = crate::game::mjolnir::shake_now();
    shake.pitch += s[0];
    shake.yaw += s[1];
    shake.roll += s[2];
    shake.offset[0] += s[3];
    shake.offset[1] += s[4];
    shake.offset[2] += s[5];
}
