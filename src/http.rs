use nom;
use std::num;
use std::fmt;
use nom::IResult;
use std::error::Error;
use crate::http::header::Header;

/// Parses [RFC7230] compliant HTTP Messages<br>
/// https://tools.ietf.org/html/rfc7230
/// <br><br>
/// # Reference
/// * `OCTET`   - any 8-bit sequence of data<br>
/// * `CHAR`    - any US-ASCII character (octets 0 - 127)<br>
/// * `UPALPHA` - any US-ASCII uppercase letter "A".."Z"<br>
/// * `LOALPHA` - any US-ASCII lowercase letter "a".."z"<br>
/// * `ALPHA`   - UPALPHA | LOALPHA<br>
/// * `DIGIT`   - any US-ASCII digit "0".."9"<br>
/// * `CTL`     - any US-ASCII control character (octets 0 - 31) and DEL (127)<br>
/// * `CR`      - US-ASCII CR, carriage return (13)<br>
/// * `LF`      - US-ASCII LF, linefeed (10)<br>
/// * `SP`      - US-ASCII SP, space (32)<br>
/// * `HT`      - US-ASCII HT, horizontal-tab (9)<br>
/// * `"`       - US-ASCII double-quote mark (34)<br>

static HEADER_NAME_MAP: [bool; 256] = byte_map![
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Parse HTTP Request Line
/// <br><br>
/// # Arguments
/// * `input` - A slice that holds the http message
/// # Expected Format
/// Method SP request-target/path SP HTTP-Version CRLF
/// <br><br>
/// https://tools.ietf.org/html/rfc7230#section-3.1.1
pub fn request_line(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8], &[u8], &[u8])> {
    nom::sequence::tuple((method, path, version, nom::character::complete::crlf))(input)
}

/// Parse HTTP Header
/// <br><br>
/// # Arguments
/// * `input` - A slice that holds the http message
/// * `header` - A mutable instance of the Header struct
/// # Expected Format
/// Header-Name: OWS Header Value OWS CRLF
/// <br><br>
/// https://tools.ietf.org/html/rfc7231#section-4
pub fn header<'i, 'h>(input: &'i [u8], header: &'h mut Header<'i>) -> nom::IResult<&'i [u8], ()> {

    let (input, name) = nom::bytes::complete::take_while(is_header_name_token)(input)?;

    header.name = name;

    let (input, _) = nom::character::complete::char(':')(input)?;

    let (input, value) = nom::sequence::delimited(
        nom::bytes::complete::tag(" "),
        nom::bytes::complete::is_not("\r\n"),
        nom::bytes::complete::tag("\r\n"),
    )(input)?;

    header.value = value;

    Ok((input, ()))
}

/// Parse HTTP Body
/// <br><br>
/// # Arguments
/// * `length` - Size of input to parse
/// * `input` - A slice that holds the http message
/// # Expected Format
/// CRLF *OCTET
/// <br><br>
/// https://tools.ietf.org/html/rfc7230#section-3.3
pub fn body(length: usize, input: &[u8]) -> nom::IResult<&[u8], &[u8]> {

    let (input, _) = nom::character::complete::crlf(input)?;

    nom::bytes::complete::take(length)(input)
}

pub fn headers_iterator<'i, 'h>(input: &'i [u8], headers: &'h mut [Header<'i>]) -> nom::IResult<&'i [u8], ()> {

    let mut iter = headers.iter_mut();
    let mut input = input;

    loop {

        let h = match iter.next() {
            Some(header) => header,
            None => break
        };

        match header(input, h) {
            Ok((i, _)) => input = i,
            Err(nom::Err::Error(_)) => break,
            e => return e
        }

    }

    Ok((input, ()))
}

fn is_header_name_token(b: u8) -> bool {
    HEADER_NAME_MAP[b as usize]
}

/// Parse HTTP request method
/// <br><br>
/// # Arguments
/// * `input` - A slice that holds the http message
/// <br><br>
/// # Expected Format
/// Any of the following: GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE, PATCH
/// <br><br>
/// https://tools.ietf.org/html/rfc7231#section-4
pub fn method(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {

    // @TODO: validate?

    // Discard CRLF if found
    let (input, _) = nom::combinator::opt(nom::character::complete::crlf)(input)?;

    // Discard numbers if found
    let (input, _) = nom::combinator::opt(nom::character::complete::digit0)(input)?;

    nom::bytes::complete::take_while_m_n(3, 7, nom::character::is_alphabetic)(input)
}

/// Parse HTTP request target
/// <br><br>
/// # Arguments
/// * `input` - A slice that holds the http message
/// <br><br>
/// # Expected Format
/// Anything that is US-ASCII SP - space (32) delimited
/// <br><br>
/// https://tools.ietf.org/html/rfc7230#section-5.3
pub fn path(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::sequence::delimited(
        nom::bytes::complete::tag(" "),
        nom::bytes::complete::is_not(" "),
        nom::bytes::complete::tag(" "),
    )(input)
}

/// Parse HTTP request protocol version
/// <br><br>
/// # Arguments
/// * `input` - A slice that holds the http message
/// # Expected Format
/// HTTP/[Version]
/// <br><br>
/// https://tools.ietf.org/html/rfc7230#section-2.6
pub fn version(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::sequence::preceded(
        nom::bytes::complete::tag("HTTP/"),
        nom::bytes::complete::take_while1(is_version),
    )(input)
}

fn is_version(input: u8) -> bool {
    input >= b'0' && input <= b'9' || input == b'.'
}

// US-ASCII SP, space (32) delimited
pub fn whitespace_delimited(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::sequence::delimited(
        nom::bytes::complete::tag(" "),
        nom::bytes::complete::is_not(" "),
        nom::bytes::complete::tag(" "),
    )(input)
}

// Not US-ASCII CR, carriage return (13) + US-ASCII LF, linefeed (10)
pub fn not_crlf(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::bytes::complete::is_not("\r\n")(input)
}

#[derive(Debug, PartialEq)]
pub enum ParserError {

    /// Represents a failure when reading HTTP Message request line.
    RequestLine,

    /// Represents a failure when reading HTTP Message headers.
    Headers,

    /// Represents a failure when reading HTTP Message body.
    Body,

    /// Represents a failure when reading HTTP Message Content Length Header
    ContentLength,

    /// Represents a failure when reading HTTP Message headers.
    InvalidUtf8Content(std::str::Utf8Error),

    /// Represents an unknown failure.
    Unknown
}

// Allow ParserError to be treated like any other error
impl Error for ParserError {}

// Allow the use of "{}" when printing ParserError
impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::RequestLine => write!(f, "ParserError: {}", "Unable to parse HTTP Message request line."),
            ParserError::Headers => write!(f, "ParserError: {}", "Unable to parse HTTP Message headers."),
            ParserError::Body => write!(f, "ParserError: {}", "Unable to parse HTTP Message body."),
            ParserError::ContentLength => write!(f, "ParserError: {}", "Unable to parse HTTP Message Content-Length header."),
            ParserError::InvalidUtf8Content(ref e) => write!(f, "ParserError: {}", e),
            ParserError::Unknown => write!(f, "ParserError: {}", "An unknown error occurred.")
        }
    }
}

// Support converting num::ParseIntError into ParserError
impl From<num::ParseIntError> for ParserError {
    fn from(_: num::ParseIntError) -> ParserError {
        ParserError::ContentLength
    }
}

// Support std::str::Utf8Error into ParserError
impl From<std::str::Utf8Error> for ParserError {
    fn from(e: std::str::Utf8Error) -> ParserError {
        ParserError::InvalidUtf8Content(e)
    }
}
