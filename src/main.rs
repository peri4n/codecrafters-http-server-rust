use std::{
    io::{BufReader, Write, BufRead},
    net::{TcpListener, TcpStream},
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
    let reader = BufReader::new(&mut stream);
    let request_line = reader.lines().next().unwrap().unwrap();

    let response = match request_line.as_str() {
        "GET / HTTP/1.1" => String::from("HTTP/1.1 200 OK\r\n\r\n"),
        s if s.starts_with("GET /echo/") => {
            let path = s[10..].split_whitespace().next().unwrap();
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", path.len(), path)
        }
        _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    };
    stream.write_all(response.as_bytes()).unwrap();
}
