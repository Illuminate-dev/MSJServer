use std::future::Future;

use axum::{body::Body, extract::Request, http::Response};
use pin_project::pin_project;
use tower::{Layer, Service};

use crate::*;

#[derive(Clone)]
pub struct AdminAuthLayer {
    state: ServerState,
}

impl AdminAuthLayer {
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for AdminAuthLayer {
    type Service = AdminAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AdminAuth {
            state: self.state.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct AdminAuth<S> {
    inner: S,
    state: ServerState,
}

impl<S> Service<Request<Body>> for AdminAuth<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, Body>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let jar = CookieJar::from_headers(req.headers());

        if let Some(name) = get_logged_in(&self.state, &jar) {
            if get_perms(&self.state, &name) == Some(Perms::Admin) {
                ResponseFuture::future(self.inner.call(req))
            } else {
                ResponseFuture::error(
                    render_with_header(
                        jar,
                        self.state.clone(),
                        NOT_AUTHOIRZED_PAGE_TEMPLATE.into(),
                    )
                    .into_response(),
                )
            }
        } else {
            ResponseFuture::error(
                render_with_header(jar, self.state.clone(), NOT_AUTHOIRZED_PAGE_TEMPLATE.into())
                    .into_response(),
            )
        }
    }
}

#[pin_project(project = ResponseFutureProj)]
pub enum ResponseFuture<F, B> {
    Future(#[pin] F),
    Error(Option<Response<B>>),
}

impl<F, B> ResponseFuture<F, B> {
    fn future(f: F) -> Self {
        Self::Future(f)
    }

    fn error(res: Response<B>) -> Self {
        Self::Error(Some(res))
    }
}

impl<F, B, E> Future for ResponseFuture<F, B>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<B>, E>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.project() {
            ResponseFutureProj::Future(f) => f.poll(cx),
            ResponseFutureProj::Error(e) => {
                let res = e.take().expect("polled after ready");
                std::task::Poll::Ready(Ok(res))
            }
        }
    }
}
