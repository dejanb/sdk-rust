use std::{cell::RefCell, rc::Rc};

use crate::binding::http::builder::Builder;
use crate::binding::{
    http::{header_prefix, SPEC_VERSION_HEADER},
    CLOUDEVENTS_JSON_HEADER,
};
use crate::event::SpecVersion;
use crate::message::{BinarySerializer, MessageAttributeValue, Result, StructuredSerializer, Error};
use std::convert::TryFrom;
use std::fmt::Debug;

macro_rules! str_to_header_value {
    ($header_value:expr) => {
        http::header::HeaderValue::from_str(&$header_value.to_string()).map_err(|e| {
            crate::message::Error::Other {
                source: Box::new(e),
            }
        })
    };
}

pub struct Serializer<T> {
    builder: Rc<RefCell<dyn Builder<T>>>,
}

impl<T> Serializer<T> {
    pub fn new<B: Builder<T> + 'static>(delegate: B) -> Serializer<T> {
        let builder = Rc::new(RefCell::new(delegate));
        Serializer { builder }
    }
}

impl<T> BinarySerializer<http::request::Request<Option<T>>> for http::request::Builder
    where T: TryFrom<Vec<u8>>,
    <T as TryFrom<Vec<u8>>>::Error: Debug,
{

    fn set_spec_version(mut self, sv: SpecVersion) -> Result<Self> {
        self = self.header(SPEC_VERSION_HEADER, &sv.to_string());
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        let key = &header_prefix(name);
        self = self.header(key, &value.to_string());
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        let key = &header_prefix(name);
        self = self.header(key, &value.to_string());
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<http::request::Request<Option<T>>> {
        let body = T::try_from(bytes).unwrap();
        self.body(Some(body)).map_err(|e| Error::Other { source: Box::new(e) })
    }

    fn end(self) -> Result<http::request::Request<Option<T>>> {
        self.body(None).map_err(|e| Error::Other { source: Box::new(e) })
    }

}

impl<T> BinarySerializer<T> for Serializer<T> {
    fn set_spec_version(self, spec_version: SpecVersion) -> Result<Self> {
        self.builder
            .borrow_mut()
            .header(SPEC_VERSION_HEADER, str_to_header_value!(spec_version)?);
        Ok(self)
    }

    fn set_attribute(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder
            .borrow_mut()
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn set_extension(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder
            .borrow_mut()
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<T> {
        self.builder.borrow_mut().body(bytes)
    }

    fn end(self) -> Result<T> {
        self.builder.borrow_mut().finish()
    }
}

impl<T> StructuredSerializer<T> for Serializer<T> {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<T> {
        let mut builder = self.builder.borrow_mut();
        builder.header(
            http::header::CONTENT_TYPE.as_str(),
            http::HeaderValue::from_static(CLOUDEVENTS_JSON_HEADER),
        );
        builder.body(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::message::BinaryDeserializer;
    use crate::test::fixtures;
    use http::Request;
    use bytes::Bytes;

    #[test]
    fn test_event_to_http_request() {
        let event = fixtures::v10::minimal_string_extension();
        let request: Request<Option<Vec<u8>>> = BinaryDeserializer::deserialize_binary(event, http::request::Builder::new()).unwrap();

        assert_eq!(request.headers()["ce-id"], "0001");
        assert_eq!(request.headers()["ce-type"], "test_event.test_application");
    }

    #[test]
    fn test_event_to_bytes_body() {
        let event = fixtures::v10::full_binary_json_data_string_extension();
        let request: Request<Option<Bytes>> = BinaryDeserializer::deserialize_binary(event, http::request::Builder::new()).unwrap();

        assert_eq!(request.headers()["ce-id"], "0001");
        assert_eq!(request.headers()["ce-type"], "test_event.test_application");
        assert_eq!(request.body().as_ref().unwrap(), &Bytes::from(fixtures::json_data().to_string()));
    }
}