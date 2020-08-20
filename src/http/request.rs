use std::fmt;
use std::str;
use crate::http::parse::body;
use crate::http::header::Header;
use crate::http::parse::ParserError;
use crate::http::parse::request_line;
use crate::http::parse::headers_iterator;

#[derive(Debug, Default)]
pub struct Request<'a> {

    /// The request method, such as `GET`.
    pub(crate) method: &'a [u8],

    /// The request path, such as `/events`.
    pub(crate) path: &'a [u8],

    /// The request version, such as `HTTP/1.1`.
    pub(crate) version: &'a [u8],

    /// The request headers, such as `Host: subdomain.domain.tld`
    pub(crate) headers: &'a mut [Header<'a>],

    /// The request body, such as `{\"dummy\": \"response\"}`
    pub(crate) body: &'a [u8]

}

impl<'i> Request<'i> {

    pub fn new(headers: &'i mut [Header<'i>]) -> Self {
        Self { headers, ..Default::default() }
    }

    pub fn method(&self) -> &[u8] {
        self.method
    }

    pub fn path(&self) -> &[u8] {
        self.path
    }

    pub fn version(&self) -> &[u8] {
        self.version
    }

    pub fn headers(&self) -> &[Header] {

        // Since `headers` is an array with a fixed size, some of its entries could be blank,
        // The parsed headers will not always fill it up completely, to remedy that, iterate over the array and return slice of
        // length 0 to fist blank entry
        let mut length = 0;

        for (i, elem) in self.headers.iter().enumerate() {
            if elem.name.len() == 0 && elem.value.len() == 0 {
                length = i;
                break;
            }
        }

        &self.headers[..length]
    }

    pub fn body(&self) -> &[u8] {
        self.body
    }

    pub fn parse<'r: 'i>(&mut self, input: &'i [u8]) -> Result<(), ParserError> {

        let mut unparsed_input;

        // Request line
        match request_line(input) {
            Ok((input, (method, path, version, _))) => {

                self.method = method;
                self.path = path;
                self.version = version;

                unparsed_input = input;
            },
            Err(_) => return Err(ParserError::RequestLine)
        };

        // Headers
        match headers_iterator(unparsed_input, self.headers) {
            Ok((input, _)) => unparsed_input = input,
            Err(_) => return Err(ParserError::Headers)
        };

        // Content
        // Check for Content-Length or Transfer-Encoding to determine if request has a body
        if let Some(header) = self.headers.iter().find(|&h| {

            // https://tools.ietf.org/html/rfc7230#section-3.3.2
            h.name() == &b"Content-Length"[..] && h.value() > &b"0"[..] || h.name() == &b"Transfer-Encoding"[..]

        }) {

            if header.name() == &b"Content-Length"[..] {

                let length = str::from_utf8(header.value())?;

                let length = length.parse::<usize>()?;

                match body(length, unparsed_input) {
                    Ok((_, body)) => {

                        self.body = body;

                    },
                    Err(_) => return Err(ParserError::Body)
                };
            }

            // @TODO: implement this
            if header.name() == &b"Transfer-Encoding"[..] {}
        }

        Ok(())
    }
}

impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let mut headers = String::new();

        let headers_length = self.headers().len();

        for (i, element) in self.headers().iter().enumerate() {

            let header;

            if i != (headers_length - 1) {

                header = format!("{}: {}, ", str::from_utf8(element.name).unwrap(), str::from_utf8(element.value).unwrap());

            } else {

                header = format!("{}: {}", str::from_utf8(element.name).unwrap(), str::from_utf8(element.value).unwrap());

            }

            headers.push_str(&header)
        };

        write!(
            f,
            "Request {{ method: {}, path: {}, version: HTTP/{}, headers: {{ {} }} }}",
            str::from_utf8(self.method).unwrap(),
            str::from_utf8(self.path).unwrap(),
            str::from_utf8(self.version).unwrap(),
            headers
        )
    }
}


