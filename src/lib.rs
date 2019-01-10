// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

#![deny(warnings)]

mod client;
mod err;
mod protocol;
mod service;

use crate::client::Client;
pub use crate::err::{Error, ErrorKind};
use crate::protocol::{Protocol, ProtocolHere, ProtocolMapQuest};
pub use crate::service::{FindLatLong, ServiceBuilder};

use serde_derive::Serialize;

/// Latitude and longitude in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}
