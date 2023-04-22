use rustified::endpoint::Endpoint;
use rustified_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(method = "POST")]
struct Test {}

fn main() {}
