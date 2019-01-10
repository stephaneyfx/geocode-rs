// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

use crate::{Client, Coordinates, Error, ErrorKind, ProtocolHere,
    ProtocolMapQuest};
use futures::{Future, Stream};
use hyper::{Body, Request, Response, Server};
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceBuilder {
    #[serde(default = "ServiceBuilder::default_sock_addr")]
    sock_addr: SocketAddr,
    #[serde(default)]
    finder: FindLatLong,
}

impl ServiceBuilder {
    pub const DEFAULT_IP: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    pub const DEFAULT_PORT: u16 = 8080;

    fn default_sock_addr() -> SocketAddr {
        (Self::DEFAULT_IP, Self::DEFAULT_PORT).into()
    }

    pub fn build(self) -> impl Future<Item = (), Error = Error> {
        let service = Service {
            finder: self.finder,
        };
        let service = Arc::new(service);
        let server = Server::bind(&self.sock_addr)
            .serve(move || {
                let service = service.clone();
                hyper::service::service_fn(move |req| {
                    reply(&service, req)
                })
            })
            .from_err();
        server
    }

    pub fn address(mut self, a: SocketAddr) -> Self {
        self.sock_addr = a;
        self
    }

    pub fn ip(mut self, a: IpAddr) -> Self {
        self.sock_addr.set_ip(a);
        self
    }

    pub fn port(mut self, p: u16) -> Self {
        self.sock_addr.set_port(p);
        self
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.sock_addr
    }

    pub fn from_config<R: Read>(config: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(config)
    }
}

#[derive(Debug)]
struct Service {
    finder: FindLatLong,
}

fn reply(service: &Service, req: Request<Body>)
    -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send>
{
    if req.method() != hyper::Method::GET {
        return async_err(ErrorKind::BadRequest.into())
    }
    let uri = req.uri();
    let req_kind = match uri.path() {
        "/geocode" => RequestKind::Geocode,
        "/map" => RequestKind::Map,
        _ => return async_err(ErrorKind::BadRequest.into()),
    };
    let suffix = uri.path_and_query().map_or("", |s| s.as_str());
    let url = format!("http://dummy.com{}", suffix);
    let url = Url::parse(&url).unwrap();
    let location = url.query_pairs()
        .find(|(k, _)| k == "location")
        .map(|(_, v)| v);
    let location = match location {
        Some(location) => location,
        None => return async_err(Error::new(ErrorKind::BadRequest,
            "Missing location parameter")),
    };
    let finder = service.finder.clone();
    let coords = finder.find(&location);
    let resp = coords
        .then(move |coords| {
            let resp = match req_kind {
                RequestKind::Geocode => respond(&coords),
                RequestKind::Map => open_map(&coords),
            };
            Ok(resp)
        });
    Box::new(resp)
}

fn async_err(e: Error)
    -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send>
{
    respond_async::<()>(&Err(e))
}

fn respond_async<T: Serialize>(r: &Result<T, Error>)
    -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send>
{
    Box::new(futures::future::ok(respond(r)))
}

fn respond<T: Serialize>(r: &Result<T, Error>) -> Response<Body> {
    let status = match r {
        Ok(_) => hyper::StatusCode::OK,
        Err(e) => match e.kind() {
            ErrorKind::BackendFailure =>
                hyper::StatusCode::SERVICE_UNAVAILABLE,
            ErrorKind::BadRequest => hyper::StatusCode::BAD_REQUEST,
            ErrorKind::LocationNotFound => hyper::StatusCode::OK,
        }
    };
    let body = Body::from(serde_json::to_string_pretty(&r).unwrap());
    Response::builder()
        .status(status)
        .header(hyper::header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(body)
        .unwrap()
}

fn open_map(coords: &Result<Coordinates, Error>) -> Response<Body> {
    let coords = match coords {
        Ok(coords) => *coords,
        Err(_) => return respond(coords),
    };
    let target = format!("https://www.google.com/maps/search/?api=1&\
        query={},{}", coords.latitude, coords.longitude);
    Response::builder().status(hyper::StatusCode::TEMPORARY_REDIRECT)
        .header("Location", target)
        .body(Body::empty())
        .unwrap()
}

#[derive(Clone, Copy, Debug)]
enum RequestKind {
    Geocode,
    Map,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum AnyProtocol {
    Here(ProtocolHere),
    MapQuest(ProtocolMapQuest),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FindLatLong {
    protocols: Vec<AnyProtocol>,
}

impl FindLatLong {
    pub fn find(self, location: &str)
        -> impl Future<Item = Coordinates, Error = Error>
    {
        let no_protocol = self.protocols.is_empty();
        let location = location.to_string();
        futures::stream::iter_ok(self.protocols)
            .and_then(move |proto| {
                match proto {
                    AnyProtocol::Here(proto) => {
                        let client = Client::new(proto);
                        client.find_lat_long(&location)
                    }
                    AnyProtocol::MapQuest(proto) => {
                        let client = Client::new(proto);
                        client.find_lat_long(&location)
                    }
                }
            })
            .then(Ok)
            .filter_map(Result::ok)
            .into_future()
            .map_err(|(e, _)| e)
            .and_then(move |(coords, _)| {
                coords.ok_or_else(|| {
                    if no_protocol {
                        Error::new(ErrorKind::LocationNotFound,
                            "No backend service configured")
                    } else {
                        Error::from(ErrorKind::LocationNotFound)
                    }
                })
            })
    }
}
