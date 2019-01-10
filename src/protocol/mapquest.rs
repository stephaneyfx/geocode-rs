// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

use crate::{Coordinates, Error, ErrorKind, Protocol};
use futures::{Future, Stream};
use hyper::{Body, Request, Response};
use serde_derive::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProtocolMapQuest {
    key: String,
}

const URL_BASE: &str = "https://www.mapquestapi.com/geocoding/v1/address";

impl Protocol for ProtocolMapQuest {
    fn request(&self, loc: &str) -> Request<Body> {
        let params = [
            ("key", self.key.as_str()),
            ("location", loc),
        ];
        let url = Url::parse_with_params(URL_BASE, &params).unwrap();
        let req = Request::builder().uri(url.as_str())
            .body(Body::empty()).unwrap();
        req
    }

    fn parse(&self, response: Response<Body>)
        -> Box<dyn Future<Item = Coordinates, Error = Error> + Send>
    {
        let coords = response.into_body()
            .concat2()
            .from_err()
            .and_then(|body| {
                let obj = serde_json::from_slice::<serde_json::Value>(&body);
                obj.map_err(|e| Error::new(ErrorKind::BackendFailure, e))
            })
            .and_then(|obj| {
                obj
                    .pointer("/results/0/locations/0/latLng")
                    .and_then(|pos| {
                        let lat = pos.get("lat").and_then(|lat| lat.as_f64());
                        let long = pos.get("lng")
                            .and_then(|long| long.as_f64());
                        lat.and_then(|lat| long.map(|long| (lat, long)))
                            .map(|(latitude, longitude)| {
                                Coordinates {latitude, longitude}
                            })
                    })
                    .ok_or_else(|| ErrorKind::LocationNotFound.into())
            });
        Box::new(coords)
    }
}
