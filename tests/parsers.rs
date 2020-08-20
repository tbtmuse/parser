use nom::Err;
use parser::http;
use nom::error::ErrorKind;
use parser::http::request::Request;

#[test]
fn test_method() {
    assert_eq!(http::parse::method(b"GET /x HTTP/1.1\r\n"), Ok((&b" /x HTTP/1.1\r\n"[..], &b"GET"[..])));
    assert_eq!(http::parse::method(b"\r\nGET /x HTTP/1.1\r\n"), Ok((&b" /x HTTP/1.1\r\n"[..], &b"GET"[..])));
    assert_eq!(http::parse::method(b"123454GET /x HTTP/1.1\r\n"), Ok((&b" /x HTTP/1.1\r\n"[..], &b"GET"[..])));
}

#[test]
fn test_path() {
    assert_eq!(
        http::parse::path(b" /RandomPath/tag.data?cn=tf&c=19&mc=imp&pli=9962555&PluID=0&ord=1400862593645&rtu=-1 HTTP/1.1\r\n"),
        Ok((
            &b"HTTP/1.1\r\n"[..],
            &b"/RandomPath/tag.data?cn=tf&c=19&mc=imp&pli=9962555&PluID=0&ord=1400862593645&rtu=-1"[..]
        ))
    );
    assert_eq!(
        http::parse::path(b" /wallpapers/hd.png?v=hOlmDALJCWWdjzfBV4ZxJPmrdCLWB%2Ftq7Z%2Ffp4Q%2FxXbVPPREuMJMVGzKraTuhhNWxCCwi6yFEZg%3D&r=783333388 HTTP/1.1\r\n"),
        Ok((
            &b"HTTP/1.1\r\n"[..],
            &b"/wallpapers/hd.png?v=hOlmDALJCWWdjzfBV4ZxJPmrdCLWB%2Ftq7Z%2Ffp4Q%2FxXbVPPREuMJMVGzKraTuhhNWxCCwi6yFEZg%3D&r=783333388"[..]
        ))
    );
    assert_eq!(http::parse::path(b" /x HTTP/1.1\r\n"), Ok((&b"HTTP/1.1\r\n"[..], &b"/x"[..])));
}

#[test]
fn test_version() {
    assert_eq!(http::parse::version(b"HTTP/1.1\r\n"), Ok((&b"\r\n"[..], &b"1.1"[..])));
    assert_eq!(http::parse::version(b"HTTP/2\r\n"), Ok((&b"\r\n"[..], &b"2"[..])));
    assert_eq!(http::parse::version(b"HTTP/3\r\n"), Ok((&b"\r\n"[..], &b"3"[..])));
}

#[test]
fn test_not_crlf() {
    assert_eq!(http::parse::not_crlf(b"abcd efg\r\n"), Ok((&b"\r\n"[..], &b"abcd efg"[..])));
    assert_eq!(http::parse::not_crlf(b"frfrfrfrfrfr\r\n"), Ok((&b"\r\n"[..], &b"frfrfrfrfrfr"[..])));
    assert_eq!(http::parse::not_crlf(b""), Err(Err::Error((&b""[..], ErrorKind::IsNot))));
}

#[test]
fn test_space_delimited() {
    assert_eq!(http::parse::whitespace_delimited(b" abcd "), Ok((&b""[..], &b"abcd"[..])));
    assert_eq!(http::parse::whitespace_delimited(b"nospace"), Err(Err::Error((&b"nospace"[..], ErrorKind::Tag))));
}

#[test]
fn test_request_without_body() {
    let data = "\
        GET / HTTP/1.1\r\n\
        Host: 127.0.0.1:9000\r\n\
        Connection: Upgrade\r\n\
        Pragma: no-cache\r\n\
        Cache-Control: no-cache\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.61 Safari/537.36\r\n\
        Upgrade: websocket\r\n\
        Origin: http://local.test.tld\r\n\
        Sec-WebSocket-Version: 13\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        Accept-Language: en-ZA,en-GB;q=0.9,en-US;q=0.8,en;q=0.7\r\n\
        Sec-WebSocket-Key: t/p5xBb6yGX25WLXAjeS0A==\r\n\
        Sec-WebSocket-Extensions: permessage-deflate; client_max_window_bits\r\n\
    ";

    let mut headers = [http::header::EMPTY_HEADER; 32];

    let mut request = Request::new(&mut headers);

    match request.parse(data.as_bytes()) {
        Ok(_) => {}
        Err(e) => panic!("Something went wrong: {:?}", e)
    }

    assert_eq!(request.method(), b"GET");
    assert_eq!(request.path(), b"/");
    assert_eq!(request.version(), b"1.1");
    assert_eq!(request.headers().len(), 12);
    assert_eq!(request.body().len(), 0);
}

#[test]
fn test_request_with_body() {
    let data = "\
        GET / HTTP/1.1\r\n\
        Host: 127.0.0.1:9000\r\n\
        Pragma: no-cache\r\n\
        Cache-Control: no-cache\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.61 Safari/537.36\r\n\
        Origin: http://local.test.tld\r\n\
        Content-Length: 16\r\n\
        Content-Type: application/json\r\n\
        \r\n\
        {\"test\": \"data\"}\
    ";

    let mut headers = [http::header::EMPTY_HEADER; 32];

    let mut request = Request::new(&mut headers);

    match request.parse(data.as_bytes()) {
        Ok(_) => {}
        Err(e) => panic!("Something went wrong: {:?}", e)
    }

    assert_eq!(request.method(), b"GET");
    assert_eq!(request.path(), b"/");
    assert_eq!(request.version(), b"1.1");
    assert_eq!(request.headers().len(), 7);
    assert_eq!(request.body().len(), 16);
    assert_eq!(request.body().len(), 16);
}

#[test]
fn test_ignores_body_if_content_length_and_transfer_encoding_header_is_absent() {
    let data = "\
        GET / HTTP/1.1\r\n\
        Host: 127.0.0.1:9000\r\n\
        Connection: Upgrade\r\n\
        Pragma: no-cache\r\n\
        Cache-Control: no-cache\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.61 Safari/537.36\r\n\
        Upgrade: websocket\r\n\
        Origin: http://local.test.tld\r\n\
        Sec-WebSocket-Version: 13\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        Accept-Language: en-ZA,en-GB;q=0.9,en-US;q=0.8,en;q=0.7\r\n\
        Sec-WebSocket-Key: t/p5xBb6yGX25WLXAjeS0A==\r\n\
        Sec-WebSocket-Extensions: permessage-deflate; client_max_window_bits\r\n\
        \r\n\
        {\"test\":\"data\"}\
    ";

    let mut headers = [http::header::EMPTY_HEADER; 32];

    let mut request = Request::new(&mut headers);

    match request.parse(data.as_bytes()) {
        Ok(_) => {}
        Err(e) => panic!("Something went wrong: {:?}", e)
    }

    assert_eq!(request.method(), b"GET");
    assert_eq!(request.path(), b"/");
    assert_eq!(request.version(), b"1.1");
    assert_eq!(request.headers().len(), 12);
    assert_eq!(request.body().len(), 0);
}

#[test]
fn test_header() {
    let mut test_header = http::header::EMPTY_HEADER;

    let (_, _) = http::parse::header(b"Host: 127.0.0.1:9000\r\n", &mut test_header).unwrap();

    assert_eq!((test_header.name(), test_header.value()), (&b"Host"[..], &b"127.0.0.1:9000"[..]))
}
