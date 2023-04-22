use rustified::endpoint::Endpoint;
use rustified_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint]
struct Test {}

#[derive(Debug, Endpoint, Serialize)]
#[endpoint()]
struct TestTwo {}

fn main() {}
