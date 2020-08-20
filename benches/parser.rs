use bencher::Bencher;
use bencher::black_box;
use parser::http::header;
use bencher::benchmark_main;
use bencher::benchmark_group;
use parser::http::request::Request;

fn test(b: &mut Bencher) {

   let data = &b"\
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
    "[..];

    b.bytes = data.len() as u64;

    parse(b, data)
}

fn parse(b: &mut Bencher, buffer: &[u8]) {

    b.iter(|| {

        let mut headers = [header::EMPTY_HEADER; 32];

        let mut req = Request::new(&mut headers);

        let buffer = black_box(buffer);

        match req.parse(&buffer) {
            Ok(_) => {},
            Err(e) => panic!("Something went wrong: {}", e)
        }

    });
}

benchmark_group!(http, test);
benchmark_main!(http);
