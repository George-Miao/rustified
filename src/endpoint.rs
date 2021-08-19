use crate::{
    client::Request,
    enums::{RequestMethod, RequestType, ResponseType},
    errors::ClientError,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use url::Url;

/// Represents a remote HTTP endpoint which can be executed using a
/// [crate::client::Client].
///
/// This trait can be implemented directly, however, users should prefer using
/// the provided `rustify_derive` macro for generating implementations. An
/// Endpoint consists of:
///   * An `action` which is combined with the base URL of a Client to form a
///     fully qualified URL.
///   * A `method` of type [RequestType] which determines the HTTP method used
///     when a Client executes this endpoint.
///   * A `ResponseType` type which determines the type of response this
///     Endpoint will return when executed.
///
/// Presently, this trait only supports sending and receiving data using JSON.
/// The struct implementing this trait must also implement [serde::Serialize].
/// The fields of the struct act as a representation of data that will be
/// serialized and sent to the remote server. Fields that should be excluded
/// from this behavior can be tagged with the `#[serde(skip)]` attribute. The
/// Endpoint will take the raw response body from the remote server and attempt
/// to deserialize it into the given `ResponseType` which must implement
/// [serde::Deserialize]. This deserialized value is then returned after
/// execution completes.
///
/// Implementations can override the default [transform][Endpoint::transform] in
/// order to modify the raw response content from the remote server before
/// returning it. This is often useful when the remote API wraps all responses
/// in a common format and the desire is to remove the wrapper before returning
/// the deserialized response. It can also be used to check for any errors
/// generated by the API and escalate them accordingly.
///
/// # Example
/// ```
/// use rustify::clients::reqwest::ReqwestClient;
/// use rustify::endpoint::Endpoint;
/// use rustify_derive::Endpoint;
/// use serde::Serialize;
///
/// #[derive(Debug, Endpoint, Serialize)]
/// #[endpoint(path = "my/endpoint")]
/// struct MyEndpoint {}
///
/// // Configure a client with a base URL of http://myapi.com
/// let client = ReqwestClient::default("http://myapi.com");
///     
/// // Construct a new instance of our Endpoint
/// let endpoint = MyEndpoint {};
///
/// // Execute our Endpoint using the client
/// // This sends a GET request to http://myapi.com/my/endpoint
/// // It assumes an empty response
/// let result = endpoint.execute(&client);
/// ```
pub trait Endpoint: Debug + Serialize + Sized {
    /// The type that the raw response from executing this endpoint will
    /// automatically be deserialized to. This type must implement
    /// [serde::Deserialize].
    type Result: DeserializeOwned;

    const REQUEST_BODY_TYPE: RequestType;
    const RESPONSE_BODY_TYPE: ResponseType;

    /// The relative URL path that represents the location of this Endpoint.
    /// This is combined with the base URL from a
    /// [Client][crate::client::Client] instance to create the fully qualified
    /// URL.
    fn action(&self) -> String;

    /// The HTTP method to be used when executing this Endpoint.
    fn method(&self) -> RequestMethod;

    /// Optional query parameters to add to the request
    fn query(&self) -> Vec<(String, Value)> {
        Vec::new()
    }

    /// Executes the Endpoint using the given [Client][crate::client::Client]
    /// and returns the deserialized response as defined by
    /// [Endpoint::Response].
    fn execute<C: crate::client::Client>(
        &self,
        client: &C,
    ) -> Result<Option<Self::Result>, ClientError> {
        log::info!("Executing endpoint");
        log::debug! {"Endpoint: {:#?}", self};

        let url = build_url(self, client.base())?;
        let method = self.method();
        let query = self.query();
        let body = match Self::REQUEST_BODY_TYPE {
            RequestType::JSON => {
                let parse_data =
                    serde_json::to_string(self).map_err(|e| ClientError::DataParseError {
                        source: Box::new(e),
                    })?;
                match parse_data.as_str() {
                    "null" => "".to_string(),
                    "{}" => "".to_string(),
                    _ => parse_data,
                }
            }
        };

        parse(
            self,
            client.execute(Request {
                url,
                method,
                query,
                body: body.into_bytes(),
            }),
        )
    }

    /// Can be overriden by implementations in order to operate on the raw
    /// response from executing this Endpoint prior to returning the final
    /// response type.
    fn transform(&self, res: String) -> Result<String, ClientError> {
        Ok(res)
    }
}

/// Combines the given base URL with the relative URL path from this
/// Endpoint to create a fully qualified URL.
fn build_url<E: Endpoint>(endpoint: &E, base: &str) -> Result<url::Url, ClientError> {
    log::info!(
        "Building endpoint url from {} base URL and {} action",
        base,
        endpoint.action()
    );

    let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError {
        url: base.to_string(),
        source: e,
    })?;
    url.path_segments_mut()
        .unwrap()
        .extend(endpoint.action().split('/'));
    Ok(url)
}

/// Parses the raw response from executing the endpoint into a response type
/// as defined by [Endpoint::Response].
fn parse<E: Endpoint>(
    endpoint: &E,
    res: Result<Vec<u8>, ClientError>,
) -> Result<Option<E::Result>, ClientError> {
    let body = res?;
    match body.is_empty() {
        false => match E::RESPONSE_BODY_TYPE {
            ResponseType::JSON => {
                let body_err = body.clone();
                let c =
                    String::from_utf8(body).map_err(|e| ClientError::ResponseConversionError {
                        source: Box::new(e),
                        content: body_err,
                    })?;
                log::info!("Parsing JSON result from string");
                log::debug!("Content before transform: {}", c);
                let c = endpoint.transform(c)?;
                log::debug!("Content after transform: {}", c);
                match c.is_empty() {
                    false => Ok(Some(serde_json::from_str(c.as_str()).map_err(|e| {
                        ClientError::ResponseParseError {
                            source: Box::new(e),
                            content: c.clone(),
                        }
                    })?)),
                    true => Ok(None),
                }
            }
        },
        true => Ok(None),
    }
}

/// Represents an empty Endpoint result.
#[derive(Deserialize, Debug)]
pub struct EmptyEndpointResult {}
