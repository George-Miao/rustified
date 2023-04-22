#![doc = include_str!("../README.md")]

#[macro_use]
extern crate tracing;

#[cfg(feature = "blocking")]
pub mod blocking;
pub mod client;
pub mod clients;
pub mod endpoint;
pub mod enums;
pub mod errors;
pub mod http;

#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

pub use crate::{
    clients::reqwest::Client,
    endpoint::{Endpoint, MiddleWare, Wrapper},
};
