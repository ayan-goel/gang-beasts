use rapier3d::prelude::*;

#[derive(Default)]
pub(crate) struct DebugLines(pub Vec<f32>);

impl DebugRenderBackend for DebugLines {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject,
        start: Point<Real>,
        end: Point<Real>,
        _color: DebugColor,
    ) {
        self.0.extend_from_slice(start.coords.as_slice());
        self.0.extend_from_slice(end.coords.as_slice());
    }
}
