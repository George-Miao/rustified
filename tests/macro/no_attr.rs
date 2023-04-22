use rustified::endpoint::Endpoint;
use rustified_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
struct Test {}

fn main() {}
