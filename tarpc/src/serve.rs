//! Shared service handler primitives.
use std::marker::PhantomData;

use crate::{RequestName, ServerError, context};

/// Equivalent to a `FnOnce(Req) -> impl Future<Output = Resp>`.
#[allow(async_fn_in_trait)]
pub trait Serve {
    /// Type of request.
    type Req: RequestName;

    /// Type of response.
    type Resp;

    /// Responds to a single request.
    async fn serve(self, ctx: context::Context, req: Self::Req) -> Result<Self::Resp, ServerError>;
}

/// A Serve wrapper around a Fn.
#[derive(Debug)]
pub struct ServeFn<Req, Resp, F> {
    f: F,
    data: PhantomData<fn(Req) -> Resp>,
}

impl<Req, Resp, F> Clone for ServeFn<Req, Resp, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            data: PhantomData,
        }
    }
}

impl<Req, Resp, F> Copy for ServeFn<Req, Resp, F> where F: Copy {}

/// Creates a [`Serve`] wrapper around a `FnOnce(context::Context, Req) -> impl Future<Output =
/// Result<Resp, ServerError>>`.
pub fn serve<Req, Resp, Fut, F>(f: F) -> ServeFn<Req, Resp, F>
where
    F: FnOnce(context::Context, Req) -> Fut,
    Fut: Future<Output = Result<Resp, ServerError>>,
{
    ServeFn {
        f,
        data: PhantomData,
    }
}

impl<Req, Resp, Fut, F> Serve for ServeFn<Req, Resp, F>
where
    Req: RequestName,
    F: FnOnce(context::Context, Req) -> Fut,
    Fut: Future<Output = Result<Resp, ServerError>>,
{
    type Req = Req;
    type Resp = Resp;

    async fn serve(self, ctx: context::Context, req: Req) -> Result<Resp, ServerError> {
        (self.f)(ctx, req).await
    }
}