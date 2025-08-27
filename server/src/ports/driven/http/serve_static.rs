use std::{convert::Infallible, path::Path, pin::pin, task::Poll};

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use http::{Method, StatusCode};
use serde_json::json;
use tower::Service;
use tower_http::services::ServeFile;

use crate::ports::driven::http::problem::ProblemBuilder;

pub struct ServeStaticFuture {
    inner: <ServeFile as Service<Request>>::Future,
    method: Method,
    path: String,
}

impl Future for ServeStaticFuture {
    type Output = Result<Response, Infallible>;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let inner = pin!(&mut self.inner);
        match inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(r) => Poll::Ready(r.map(|r| {
                if r.status() == StatusCode::METHOD_NOT_ALLOWED {
                    ProblemBuilder::METHOD_NOT_ALLOWED
                        .detail(format!(
                            "Method {} not allowed for route {}.",
                            self.method, self.path,
                        ))
                        .with_instance(self.path.clone())
                        .with_extension("method".into(), json!(self.method.as_str()))
                        .into_response()
                } else {
                    r.map(Body::new)
                }
            })),
        }
    }
}

#[derive(Clone)]
pub struct ServeStaticService {
    inner: ServeFile,
}

impl ServeStaticService {
    pub fn new<A: AsRef<Path>>(path: A) -> Self {
        Self {
            inner: ServeFile::new(path),
        }
    }
}

impl Service<Request> for ServeStaticService {
    type Error = Infallible;
    type Response = Response;
    type Future = ServeStaticFuture;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let method = request.method().clone();
        let path = request.uri().path().to_owned();
        ServeStaticFuture {
            inner: self.inner.call(request),
            method,
            path,
        }
    }
}
