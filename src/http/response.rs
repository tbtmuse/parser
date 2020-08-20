use std::str;
use std::fmt;
use std::ops::Add;
use crate::http::header::Header;

#[derive(Debug, Default)]
pub struct Response<'a> {
    /// The response version, such as `HTTP/1.1`.
    pub version: &'a [u8],

    /// The response status code, such as `200`.
    pub status: u16,

    /// The response reason-phrase, such as `OK`.
    pub reason: &'a [u8],

    /// The response headers, such as `Accept: application/json`.
    pub headers: Vec<Header<'a>>,

    /// The response body, such as `{\"dummy\": \"response\"}`
    pub body: &'a [u8],
}

impl<'a> Response<'a> {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn version(&self) -> &[u8] {
        self.version
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn reason(&self) -> &[u8] {
        self.reason
    }

    pub fn headers(&self) -> Vec<Header> {
        self.headers.to_owned()
    }

    pub fn body(&self) -> &[u8] {
        self.body
    }
}

impl<'a> fmt::Display for Response<'a> {
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
            "Response {{ version: HTTP/{}, status: {}, reason: {}, headers: {{ {} }}, body: {{ {} }} }}",
            str::from_utf8(self.version).unwrap(),
            self.status,
            str::from_utf8(self.reason).unwrap(),
            headers,
            str::from_utf8(self.body).unwrap()
        )
    }
}

impl Into<String> for Response<'_> {
    fn into(self) -> String {

        let headers: String = self.headers()
            .into_iter()
            .map(|h| [str::from_utf8(h.name).unwrap(), str::from_utf8(h.value).unwrap()].join(": "))
            .collect::<Vec<String>>()
            .join("\r\n");

        let content = str::from_utf8(self.body).unwrap();

        let mut result= "HTTP/".to_string();

        result = result.add(&[str::from_utf8(self.version).unwrap(), self.status.to_string().as_str(), str::from_utf8(self.reason).unwrap()].join(" "));

        result = result.add("\r\n");

        if ! headers.is_empty() {
            result = result.add(&headers).add("\r\n\r\n");
        }

        if ! content.is_empty() {
            result = result.add(&content);
        }

        result
    }
}
