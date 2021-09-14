use tower::Layer;

#[derive(Clone)]
pub struct NoopLayer;

impl NoopLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for NoopLayer {
    type Service = S;

    fn layer(&self, inner: S) -> Self::Service {
        inner
    }
}
