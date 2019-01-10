// Copyright (C) 2018 Stephane Raux. Distributed under the MIT license.

use serde::Serializer;
use serde::ser::SerializeSeq;
use serde_derive::{Serialize};
use std::error::Error as StdError;
use std::fmt::{Display, self};

#[derive(Debug, Serialize)]
pub struct Error {
    kind: ErrorKind,
    #[serde(serialize_with = "serialize_cause")]
    cause: Option<Box<dyn StdError + Send + Sync>>,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, cause: E) -> Self
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        let cause = Some(cause.into());
        Error {kind, cause}
    }

    pub fn kind(&self) -> &ErrorKind {&self.kind}
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ErrorKind {
    BackendFailure,
    BadRequest,
    LocationNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::BackendFailure => f.write_str("Backend failure"),
            ErrorKind::BadRequest => f.write_str("Bad request"),
            ErrorKind::LocationNotFound => f.write_str("Location not found"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause.as_ref().map(|e| &**e as &dyn StdError)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {kind, cause: None}
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Error {
        Error::new(ErrorKind::BackendFailure, e)
    }
}

impl From<hyper_tls::Error> for Error {
    fn from(e: hyper_tls::Error) -> Error {
        Error::new(ErrorKind::BackendFailure, e)
    }
}

fn serialize_cause<S>(e: &Option<Box<dyn StdError + Send + Sync>>, out: S)
    -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = out.serialize_seq(None)?;
    let mut e = e.as_ref().map(|e| &**e as &dyn StdError);
    while let Some(cause) = e {
        seq.serialize_element(&cause.to_string())?;
        e = cause.source();
    }
    seq.end()
}
