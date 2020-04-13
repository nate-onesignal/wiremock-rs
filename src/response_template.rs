use http_types::headers::{HeaderName, HeaderValue};
use http_types::{Response, StatusCode};
use std::collections::HashMap;
use std::convert::TryInto;

/// The blueprint for the response returned by a [`MockServer`] when a [`Mock`] matches on an incoming request.
///
/// [`Mock`]: struct.Mock.html
/// [`MockServer`]: struct.MockServer.html
#[derive(Clone, PartialEq, Debug)]
pub struct ResponseTemplate {
    status_code: StatusCode,
    headers: HashMap<HeaderName, Vec<HeaderValue>>,
    body: Option<Vec<u8>>,
}

// `wiremock` is a crate meant for testing - failures are most likely not handled/temporary mistakes.
// Hence we prefer to panic and provide an easier API than to use `Result`s thus pushing
// the burden of "correctness" (and conversions) on the user.
//
// All methods try to accept the widest possible set of inputs and then perform the fallible conversion
// internally, bailing if the fallible conversion fails.
//
// Same principle applies to allocation/cloning, freely used where convenient.
impl ResponseTemplate {
    /// Start building a `ResponseTemplate` specifying the status code of the response.
    pub fn new<S>(s: S) -> Self
    where
        S: TryInto<StatusCode>,
        <S as TryInto<StatusCode>>::Error: std::fmt::Debug,
    {
        let status_code = s.try_into().expect("Failed to convert into status code.");
        Self {
            status_code,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Append a header `value` to list of headers with `key` as header name.
    ///
    /// Unlike `insert_header`, this function will not override the contents of a header:
    /// - if there are no header values with `key` as header name, it will insert one;
    /// - if there are already some values with `key` as header name, it will append to the
    ///   existing list.
    pub fn append_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        <K as TryInto<HeaderName>>::Error: std::fmt::Debug,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: std::fmt::Debug,
    {
        let key = key.try_into().expect("Failed to convert into header name.");
        let value = value
            .try_into()
            .expect("Failed to convert into header value.");
        match self.headers.get_mut(&key) {
            Some(headers) => {
                headers.push(value);
            }
            None => {
                self.headers.insert(key, vec![value]);
            }
        }
        self
    }

    /// Insert a header `value` with `key` as header name.
    ///
    /// This function will override the contents of a header:
    /// - if there are no header values with `key` as header name, it will insert one;
    /// - if there are already some values with `key` as header name, it will drop them and
    ///   start a new list of header values, containing only `value`.
    pub fn insert_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        <K as TryInto<HeaderName>>::Error: std::fmt::Debug,
        V: TryInto<HeaderValue>,
        <V as TryInto<HeaderValue>>::Error: std::fmt::Debug,
    {
        let key = key.try_into().expect("Failed to convert into header name.");
        let value = value
            .try_into()
            .expect("Failed to convert into header value.");
        self.headers.insert(key, vec![value]);
        self
    }

    /// Set the response body.
    pub fn set_body<B>(mut self, body: B) -> Self
    where
        B: TryInto<Vec<u8>>,
        <B as TryInto<Vec<u8>>>::Error: std::fmt::Debug,
    {
        let body = body.try_into().expect("Failed to convert into body.");
        self.body = Some(body);
        self
    }

    /// Generate a response from the template.
    pub(crate) fn generate_response(&self) -> Response {
        let mut response = Response::new(self.status_code);

        // Add headers
        for (header_name, header_values) in &self.headers {
            response
                .insert_header(header_name.clone(), header_values.as_slice())
                .unwrap();
        }

        // Add body, if specified
        if let Some(body) = &self.body {
            response.set_body(body.clone())
        }

        response
    }
}