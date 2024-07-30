use std::{
    io::{Write, Read},
    net::{TcpListener, TcpStream},
};

use nom::{combinator::map, bytes::complete::is_not, multi::count};
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::crlf,
    multi::separated_list0,
    sequence::{separated_pair, terminated},
    IResult, Parser,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();

    match HttpRequest::from_str(std::str::from_utf8(&buf).unwrap()) {
        HttpRequest { method: "GET", path: "/", headers: _, body: _ } => {
            write_response(stream, "200 OK", "");
        },
        req if req.path.starts_with("/echo/") => {
            write_response(stream, "200 OK", &req.path[6..]);
        },
        req if req.path.starts_with("/user-agent") => {
            write_response(stream, "200 OK", &req.headers.iter().find(|(k, _)| *k == "User-Agent").unwrap().1);
        },
        _ => {
            write_response(stream, "404 Not Found", "");
        }
    }
    //let response = match request_line.as_str() {
    //    "GET / HTTP/1.1" => String::from("HTTP/1.1 200 OK\r\n\r\n"),
    //    s if s.starts_with("GET /echo/") => {
    //        let path = s[10..].split_whitespace().next().unwrap();
    //        format!(
    //            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
    //            path.len(),
    //            path
    //        )
    //    }
    //    _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    //};
}

fn write_response(mut stream: TcpStream, status: &str, payload: &str) {
    stream.write_all(format!("HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", status, payload.len(), payload).as_bytes()).unwrap();
}

#[derive(Debug)]
struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: &'a str,
}

impl<'a> HttpRequest<'a> {
    fn parse_request_line(request_line: &str) -> IResult<&str, (&str, &str)> {
        map(terminated(take_until("\r\n"), crlf), |line: &str| {
            let mut words = line.split_whitespace();
            (words.next().unwrap(), words.next().unwrap())
        })(request_line)
    }

    fn parse_headers(headers: &str) -> IResult<&str, Vec<(&str, &str)>> {
        separated_list0(crlf, Self::parse_header)(headers)
    }

    fn parse_header(header: &str) -> IResult<&str, (&str, &str)> {
        separated_pair(is_not(":"), tag(": "), is_not("\r\n"))(header)
    }

    fn parse_body(body: &str) -> IResult<&str, &str> {
        Ok(("", body))
    }

    pub fn from_str(request: &'a str) -> Self {
        map(
            Self::parse_request_line
                .and(terminated(Self::parse_headers, count(crlf, 2)))
                .and(Self::parse_body),
            |(((method, path), headers), body)| HttpRequest {
                method,
                path,
                headers,
                body,
            },
        )(request).unwrap().1
    }
}

#[cfg(test)]
mod test {
    use crate::HttpRequest;

    #[test]
    fn parse_request_line() {
        let input = "GET / HTTP/1.1\r\n";
        let expected = ("GET", "/");
        assert_eq!(HttpRequest::parse_request_line(input), Ok(("", expected)));
    }

    #[test]
    fn parse_header() {
        let input = "Host: localhost:4221";
        let expected = ("Host", "localhost:4221");
        assert_eq!(HttpRequest::parse_header(input), Ok(("", expected)));
    }

    #[test]
    fn parse_headers() {
        let input = "Host: localhost:4221\r\nUser-Agent: curl/7.68.0\r\nAccept: */*";
        let expected = vec![
            ("Host", "localhost:4221"),
            ("User-Agent", "curl/7.68.0"),
            ("Accept", "*/*"),
        ];
        assert_eq!(HttpRequest::parse_headers(input), Ok(("", expected)));
    }

}
