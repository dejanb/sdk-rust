pub mod builder;
pub mod deserializer;
mod headers;

use crate::{
    message::{Error, MessageDeserializer},
    Event,
};
use deserializer::Deserializer;
pub use headers::Headers;
mod serializer;

pub use builder::Builder;
pub use serializer::Serializer;
use http::Response;
use std::convert::TryInto;
use std::fmt::Debug;

pub static SPEC_VERSION_HEADER: &str = "ce-specversion";

/// Turn a pile of HTTP headers and a body into a CloudEvent
pub fn to_event<'a, T: Headers<'a>>(
    headers: &'a T,
    body: Vec<u8>,
) -> std::result::Result<Event, Error> {
    MessageDeserializer::into_event(Deserializer::new(headers, body))
}

/// Method to transform an incoming [`Response`] to [`Event`].
pub async fn response_to_event<T>(res: Response<T>) -> Result<Event, Error>
    where T: TryInto<Vec<u8>>,
    <T as TryInto<Vec<u8>>>::Error: Debug,
{
    let headers = res.headers().to_owned();
    let body = T::try_into(res.into_body()).unwrap();

    to_event(&headers, body)
}

pub fn header_prefix(name: &str) -> String {
    super::header_prefix("ce-", name)
}
