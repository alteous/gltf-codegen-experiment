// support/camera.rs

/// A camera's projection.
pub enum Projection<'a> {
    /// Perspective projection.
    Perspective(::camera::Perspective<'a>),
    /// Orthographic projection.
    Orthographic(::camera::Orthographic<'a>),
}

impl<'a> Camera<'a> {
    /// Returns the camera's projection properties.
    pub fn projection(&self) -> Projection<'a> {
        match self.kind() {
            Kind::Perspective => Projection::Perspective(
                ::camera::Perspective::new(self.document, self.perspective.as_ref().unwrap())
            ),
            camera::Kind::Orthographic => Projection::Orthographic(
                ::camera::Orthographic::new(self.document, self.orthographic.as_ref().unwrap())
            ),
        }
    }
}
