use axum::body::{box_body, BoxBody};
use bytes::Bytes;
use futures_util::future::Either;
use http::{request::Request, response::Response, status::StatusCode};
use http_body::Body as HttpBody;
use hyper::body::Body;
use std::{
    convert::Infallible,
    marker::PhantomData,
    task::{Context, Poll},
};
use tower::{BoxError, ServiceExt};
use tower_layer::Layer;
use tower_service::Service;

mod box_service;

use box_service::BoxServiceFuture;

type BoxService<B, E> = box_service::BoxService<Request<B>, Response<BoxBody>, E>;

pub struct Router<S, B = Body, E = Infallible> {
    service: S,
    routes: Vec<Route<B, E>>,
    recognizer: route_recognizer::Router<usize>,
}

impl<B, E> Router<RouterService<E>, B, E> {
    pub fn new() -> Self {
        Self {
            service: RouterService::new(),
            routes: Vec::new(),
            recognizer: route_recognizer::Router::new(),
        }
    }
}

impl<S, B, E> Router<S, B, E>
where
    S: Clone,
{
    pub fn route<Svc, ResBody>(mut self, path: &str, service: Svc) -> Self
    where
        Svc: Service<Request<B>, Response = Response<ResBody>, Error = E>
            + Clone
            + Send
            + Sync
            + 'static,
        Svc::Future: Send + 'static,
        ResBody: HttpBody<Data = Bytes> + Send + Sync + 'static,
        ResBody::Error: Into<BoxError>,
    {
        self.recognizer.add(&path, self.routes.len());
        self.routes.push(Route::new(path, service));
        self
    }

    pub fn layer<L>(self, layer: L) -> Router<Layered<L::Service>, B, E>
    where
        L: Layer<S>,
    {
        let index = self.routes.len();

        self.map(|svc| Layered::new(index, layer.layer(svc)))
    }

    fn map<F, Svc>(self, f: F) -> Router<Svc, B, E>
    where
        F: FnOnce(S) -> Svc,
    {
        Router {
            service: f(self.service),
            routes: self.routes,
            recognizer: self.recognizer,
        }
    }
}

impl<S, B, E> Clone for Router<S, B, E>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            routes: self.routes.clone(),
            recognizer: self.recognizer.clone(),
        }
    }
}

impl<S, B, E> Service<Request<B>> for Router<S, B, E>
where
    S: Service<Request<B>, Response = Response<BoxBody>, Error = E>,
    B: 'static,
    E: 'static,
{
    type Response = Response<BoxBody>;
    type Error = E;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let path = req.uri().path();

        if let Ok(matched) = self.recognizer.recognize(path) {
            let index = **matched.handler();
            let service = self.routes[index].service.clone();

            let route_extension = RouteExtension::new(index, service);

            req.extensions_mut().insert(route_extension);
        }

        self.service.call(req)
    }
}

pub struct RouterService<E> {
    _marker: PhantomData<E>,
}

impl<E> RouterService<E> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E> Clone for RouterService<E> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<B, E> Service<Request<B>> for RouterService<E>
where
    B: 'static,
    E: 'static,
{
    type Response = Response<BoxBody>;
    type Error = E;
    type Future = BoxServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let route_extension = req.extensions_mut().remove::<RouteExtension<B, E>>();

        if let Some(mut route_extension) = route_extension {
            route_extension.service.call(req)
        } else {
            BoxServiceFuture::new(async {
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(box_body(Body::empty()))
                    .unwrap();

                Ok(resp)
            })
        }
    }
}

struct Route<B, E> {
    path: String,
    service: BoxService<B, E>,
}

impl<B, E> Route<B, E> {
    fn new<S, ResBody>(path: &str, service: S) -> Self
    where
        S: Service<Request<B>, Response = Response<ResBody>, Error = E>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
        ResBody: HttpBody<Data = Bytes> + Send + Sync + 'static,
        ResBody::Error: Into<BoxError>,
    {
        let service = service.map_response(|resp| resp.map(box_body));

        Self {
            path: path.to_owned(),
            service: BoxService::new(service),
        }
    }
}

impl<B, E> Clone for Route<B, E> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            service: self.service.clone(),
        }
    }
}

struct RouteExtension<B, E> {
    index: usize,
    service: BoxService<B, E>,
}

impl<B, E> RouteExtension<B, E> {
    fn new(index: usize, service: BoxService<B, E>) -> Self {
        Self { index, service }
    }
}

#[derive(Clone)]
pub struct Layered<S> {
    index: usize,
    svc: S,
}

impl<S> Layered<S> {
    fn new(index: usize, svc: S) -> Self {
        Self { index, svc }
    }
}

impl<S, B, E> Service<Request<B>> for Layered<S>
where
    S: Service<Request<B>, Response = Response<BoxBody>, Error = E>,
    B: 'static,
    E: 'static,
{
    type Response = Response<BoxBody>;
    type Error = E;
    type Future = Either<S::Future, BoxServiceFuture<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let route_extension = req.extensions().get::<RouteExtension<B, E>>();

        if let Some(route_extension) = route_extension {
            if route_extension.index < self.index {
                return Either::Left(self.svc.call(req));
            }
        } else {
            return Either::Left(self.svc.call(req));
        }

        Either::Right(RouterService::new().call(req))
    }
}
