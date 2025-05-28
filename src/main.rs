use std::{fs, io};
use std::fmt::format;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::net::Shutdown::Write as ShutdownWrite;
use crate::Method::{GET, POST};

struct Request<'a> {
    pub method: Method,
    pub target : String,
    pub protocol : String,
    pub data : Option<String>,
    pub accept : Vec<&'a str>,
    pub doc_type : String

}
impl Default for Request<'_>{
    fn default() -> Self {
        Self {
            method: GET,
            target : "/website/index.html".to_string(),
            protocol : "HTTP/1.1".to_string(),
            data : None,
            accept : vec!["text/html"],
            doc_type: "document".to_string(),
        }
    }
}
#[derive(PartialOrd, PartialEq)]
enum Method {
    GET,
    POST
}

impl Method {
    fn parse(string: &str) -> Self {
        if string.to_lowercase() == "post" {
            POST
        }
        else {
            GET
        }
    }
}

fn handle_client(mut stream: TcpStream) {

    let mut buf = [0u8;1024];

    stream.read(&mut buf).expect("Failed to read");

    let mut raw_request = String::from_utf8_lossy(&buf[..]);
    
    println!("Request : \n{}", raw_request);

    let mut request = Request::default();

    let parts : Vec<&str>= raw_request.lines().next().unwrap().split_ascii_whitespace().collect();

    request.method = Method::parse(parts.get(0).unwrap());
    request.target = parts.get(1).unwrap().to_string();
    request.protocol = parts.get(2).unwrap().to_string();

    for hdrs in raw_request.lines() {
        if hdrs.starts_with("data") {

            let data : &str = hdrs.split_once("=").unwrap().1;
            request.data = Some(data.to_string());

        }
        else if hdrs.starts_with("Accept:") {
            let trim = hdrs.strip_prefix("Accept: ").unwrap();
            request.accept  = trim.split(",").collect();
        }
        else if hdrs.starts_with("Sec-Fetch-Dest") {
            request.doc_type = hdrs.strip_prefix("Sec-Fetch-Dest: ").unwrap().to_string()
        }
    }

    if request.method == POST{
        if request.data.is_some() {
            println!("{}", request.data.unwrap())
        }
    }

    let content = if request.target.ends_with(".css") {
        "text/css"
    }
    else if request.target.ends_with(".png") {
        "image/png"
    }
    else {
        "text/html"
    };
    
    if request.doc_type == "image" {

        let image = fs::read(format!(".{}", request.target)).expect("Failed to read image");
        let mut head = format!("HTTP/1.1 200 OK\r\nServer: Rust TCP server\r\nContent-Type: {}\r\n\r\n", content);
        let response = [head.as_bytes(), image.as_slice()].concat();

        stream.write_all(response.as_slice()).expect("Write failed!");
        stream.flush().unwrap()

    }
    else {
        if request.target.ends_with(".png") {
            let mut response = format!("HTTP/1.1 200 OK\r\nServer: Rust TCP server\r\nContent-Type: text/html\r\n\r\n<body style=\"background: #121212;\"><img src=\"{}\"></img></body>",request.target);
            stream.write_all(response.as_bytes()).expect("Write failed!");
            stream.flush().unwrap()
        }
        else {
            let body = fs::read_to_string(format!(".{}", request.target)).expect("Failed to read body");
            let mut response = format!("HTTP/1.1 200 OK\r\nServer: Rust TCP server\r\nContent-Type: {}\r\n\r\n{}",content,body);
            stream.write_all(response.as_bytes()).expect("Write failed!");
            stream.flush().unwrap()
        }
    }
}


fn main() -> io::Result<()> {

    let mut listener = TcpListener::bind("localhost:80");

    for stream in listener?.incoming() {
        handle_client(stream?)
    }


    Ok(())
}