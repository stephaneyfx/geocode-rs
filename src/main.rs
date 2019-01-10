// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

#![deny(warnings)]

use clap::{App, Arg};
use futures::Future;
use std::error::Error;
use std::fmt::{Display, self};
use std::fs::File;
use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::path::Path;
use std::sync::{Arc, Mutex};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn run() -> Result<(), AppError> {
    let address_help = format!("IP address to start service on (default: {})",
        geocode::ServiceBuilder::DEFAULT_IP);
    let port_help = format!("Port to start service on (default: {})",
        geocode::ServiceBuilder::DEFAULT_PORT);
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .author(APP_AUTHORS)
        .about("Geocoding proxy service")
        .arg(
            Arg::with_name("ADDRESS")
                .short("a")
                .long("address")
                .takes_value(true)
                .help(&address_help)
        )
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .takes_value(true)
                .help(&port_help)
        )
        .arg(
            Arg::with_name("CONFIG")
                .short("c")
                .long("config")
                .takes_value(true)
                .required(true)
                .help("Path to configuration file")
        )
        .get_matches();
    let config_path = Path::new(matches.value_of("CONFIG").unwrap());
    let config_file = File::open(config_path)
        .map_err(AppError::FailedToOpenConfigFile)?;
    let mut service = geocode::ServiceBuilder::from_config(config_file)
        .map_err(AppError::BadConfigFile)?;
    if let Some(ip) = matches.value_of("ADDRESS") {
        service = service.ip(ip.parse().map_err(AppError::BadAddress)?);
    }
    if let Some(port) = matches.value_of("PORT") {
        service = service.port(port.parse().map_err(AppError::BadPort)?);
    }
    println!("Geocoding service starting on {}", service.get_address());
    let result = Arc::new(Mutex::new(Ok(())));
    hyper::rt::run({
        let res = result.clone();
        service
            .build()
            .map_err(move |e| {
                *res.lock().unwrap() = Err(AppError::ServiceError(e));
            })
    });
    let result = std::mem::replace(&mut *result.lock().unwrap(), Ok(()));
    result
}

fn main() {
    let code = if let Err(e) = run() {
        print_error(e);
        1
    } else {
        0
    };
    std::process::exit(code)
}

fn print_error(e: AppError) {
    eprintln!("Error: {}", e);
    let mut e: &dyn Error = &e;
    while let Some(cause) = e.source() {
        eprintln!("Because: {}", cause);
        e = cause;
    }
}

#[derive(Debug)]
enum AppError {
    BadAddress(AddrParseError),
    BadConfigFile(serde_json::Error),
    BadPort(ParseIntError),
    FailedToOpenConfigFile(io::Error),
    ServiceError(geocode::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::BadAddress(_) => f.write_str("Bad address"),
            AppError::BadConfigFile(_) => f.write_str("Bad configuration file"),
            AppError::BadPort(_) => f.write_str("Bad port"),
            AppError::FailedToOpenConfigFile(_) =>
                f.write_str("Failed to open configuration file"),
            AppError::ServiceError(_) => f.write_str("Service error"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::BadAddress(e) => Some(e),
            AppError::BadConfigFile(e) => Some(e),
            AppError::BadPort(e) => Some(e),
            AppError::FailedToOpenConfigFile(e) => Some(e),
            AppError::ServiceError(e) => Some(e),
        }
    }
}
