use http::{request::Request, response::Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TestMiddlewareLayer {
    append: String,
}

impl TestMiddlewareLayer {
    pub fn new(s: &str) -> Self {
        Self {
            append: s.to_owned(),
        }
    }
}

impl<S> Layer<S> for TestMiddlewareLayer {
    type Service = TestMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TestMiddleware::new(inner, self.append.clone())
    }
}

#[derive(Clone)]
pub struct TestMiddleware<S> {
    inner: S,
    append: String,
}

impl<S> TestMiddleware<S> {
    fn new(inner: S, append: String) -> Self {
        Self { inner, append }
    }
}

impl<S, B> Service<Request<B>> for TestMiddleware<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        match req.extensions_mut().get_mut::<Vec<String>>() {
            Some(vec) => {
                vec.push(self.append.clone());
            }
            None => {
                let vec = vec![self.append.clone()];
                req.extensions_mut().insert(vec);
            }
        }

        self.inner.call(req)
    }
}

#[derive(Clone)]
pub struct NotFoundLayer;

impl NotFoundLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for NotFoundLayer {
    type Service = NotFoundMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NotFoundMiddleware::new(inner)
    }
}

#[derive(Clone)]
pub struct NotFoundMiddleware<S> {
    inner: S,
}

impl<S> NotFoundMiddleware<S> {
    fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, B, ResBody> Service<Request<B>> for NotFoundMiddleware<S>
where
    S: Service<Request<B>, Response = Response<ResBody>>,
    S::Future: Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let fut = self.inner.call(req);

        Box::pin(async {
            match fut.await {
                Ok(mut resp) => {
                    *resp.status_mut() = http::status::StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(resp)
                }
                Err(e) => Err(e),
            }
        })
    }
}
