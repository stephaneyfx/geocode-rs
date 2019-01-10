// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

mod here;
mod mapquest;

pub(crate) use self::here::ProtocolHere;
pub(crate) use self::mapquest::ProtocolMapQuest;

use crate::{Coordinates, Error};
use futures::Future;
use hyper::{Body, Request, Response};

pub(crate) trait Protocol {
    fn request(&self, loc: &str) -> Request<Body>;
    fn parse(&self, response: Response<Body>)
        -> Box<dyn Future<Item = Coordinates, Error = Error> + Send>;
}
