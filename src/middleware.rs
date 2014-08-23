use std::sync::Arc;
use error::Error;

use super::request::Request;
use super::response::Response;

use super::IronResult;

pub trait Handler: Send + Sync {
    fn call(&self, &mut Request) -> IronResult<Response>;

    fn catch(&self, &mut Request, Box<Error>) -> (Response, IronResult<()>);
}

impl Handler for fn(&mut Request) -> IronResult<Response> {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        self.call(req)
    }

    fn catch(&self, _: &mut Request, err: Box<Error>) -> (Response, IronResult<()>) {
        // FIXME: Make Response a 500
        (Response, Err(err))
    }
}

pub trait BeforeMiddleware: Send + Sync {
    fn before(&self, &mut Request) -> IronResult<()>;

    fn catch(&self, _: &mut Request, err: Box<Error>) -> IronResult<()> {
        Err(err)
    }
}

pub trait AfterMiddleware: Send + Sync {
    fn after(&self, &mut Request, &mut Response) -> IronResult<()>;

    // This response was generated by the `catch` function of Handlers and is abnormal in some way.
    fn catch(&self, _: &mut Request, _: &mut Response, err: Box<Error>) -> IronResult<()> {
        Err(err)
    }
}

pub trait AroundMiddleware: Handler {
    fn with_handler(&mut self, handler: Box<Handler + Send + Sync>);
}

pub trait Chain: Handler {
    fn new<H: Handler>(H) -> Self;

    fn link<B, A>(&mut self, (B, A)) where A: AfterMiddleware, B: BeforeMiddleware;

    fn link_before<B>(&mut self, B) where B: BeforeMiddleware;

    fn link_after<A>(&mut self, A) where A: AfterMiddleware;
}

pub struct DefaultChain {
    befores: Vec<Box<BeforeMiddleware + Send + Sync>>,
    afters: Vec<Box<AfterMiddleware + Send + Sync>>,
    handler: Box<Handler + Send + Sync>
}

impl Chain for DefaultChain {
    fn new<H: Handler>(handler: H) -> DefaultChain {
        DefaultChain {
            befores: vec![],
            afters: vec![],
            handler: box handler as Box<Handler + Send + Sync>
        }
    }

    fn link<B, A>(&mut self, link: (B, A))
    where A: AfterMiddleware, B: BeforeMiddleware {
        let (before, after) = link;
        self.befores.push(box before as Box<BeforeMiddleware + Send + Sync>);
        self.afters.push(box after as Box<AfterMiddleware + Send + Sync>);
    }

    fn link_before<B>(&mut self, before: B)
    where B: BeforeMiddleware {
        self.befores.push(box before as Box<BeforeMiddleware + Send + Sync>);
    }

    fn link_after<A>(&mut self, after: A)
    where A: AfterMiddleware {
        self.afters.push(box after as Box<AfterMiddleware + Send + Sync>);
    }
}

