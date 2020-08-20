/// Empty header used to initialize headers
pub const EMPTY_HEADER: Header<'static> = Header { name: b"", value: b"" };

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Header<'a> {

    /// The name portion of a header, such as `Host`
    ///
    /// A header name must be valid US-ASCII.
    pub name: &'a [u8],

    /// The value portion of a header, such as `subdomain.domain.tld`.
    ///
    /// While headers *should* be US-ASCII, the specification allows for
    /// values that may not be, and so the value is stored as bytes.
    pub value: &'a [u8]
}

impl<'a> Header<'a> {

    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn name(&self) -> &[u8] {
        self.name
    }

    pub fn value(&self) -> &[u8] {
        self.value
    }
}
