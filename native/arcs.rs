// Client: draws the lightning arcs from the hammer to its targets, in world coordinates.
fn arc_lines(primitives: &mut WorldPrimitives) {
    for (from, to, argb, width) in crate::game::mjolnir::arc_lines() {
        primitives.line(from, to, argb, width).through_walls();
    }
}
