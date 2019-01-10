// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

use crate::{Coordinates, Error, Protocol};
use futures::Future;
use hyper_tls::HttpsConnector;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct Client<P> {
    protocol: Arc<P>,
}

impl<P: Protocol + Send + Sync + 'static> Client<P> {
    pub(crate) fn new(protocol: P) -> Self {
        let protocol = Arc::new(protocol);
        Client {protocol}
    }

    pub(crate) fn find_lat_long(&self, loc: &str)
        -> Box<dyn Future<Item = Coordinates, Error = Error> + Send>
    {
        let connector = match HttpsConnector::new(1) {
            Ok(connector) => connector,
            Err(e) => return Box::new(futures::future::err(e.into())),
        };
        let client = hyper::Client::builder()
            .build::<_, hyper::Body>(connector);
        let response = client.request(self.protocol.request(loc));
        let proto = self.protocol.clone();
        let coords = response
            .from_err()
            .and_then(move |resp| proto.parse(resp));
        Box::new(coords)
    }
}
