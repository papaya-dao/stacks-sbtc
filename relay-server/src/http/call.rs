use super::{Request, Response};

pub trait Call {
    fn call(&mut self, request: Request) -> Response;
}
