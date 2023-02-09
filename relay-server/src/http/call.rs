use std::io::Error;

use super::{Request, Response};

pub trait Call {
    fn call(&mut self, request: Request) -> Result<Response, Error>;
}
