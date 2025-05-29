use std::{env, fs, io};
use std::fmt::format;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::net::Shutdown::Write as ShutdownWrite;
use dotenv::dotenv;
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

fn handle_client(mut stream: TcpStream, chat: &mut Vec<String>) {

    let mut buf = [0u8;1024];

    stream.read(&mut buf).expect("Failed to read");

    let mut raw_request = String::from_utf8_lossy(&buf[..]);

    //println!("Request : \n{}", raw_request);

    if raw_request.is_empty() {
        return;
    }

    let mut request = Request::default();

    let parts : Vec<&str>= raw_request.lines().next().unwrap().split_ascii_whitespace().collect();

    if parts.get(1).is_none() {
        return;
    }
    let raw_target = parts.get(1).unwrap().to_string();
    
    request.method = Method::parse(parts.get(0).unwrap());
    request.target = if !raw_target.starts_with("/website"){ format!("/website{}",raw_target)} else { raw_target };
    request.protocol = parts.get(2).unwrap().to_string();

    if request.target == "/website/" {
        request.target = "/website/index.html".to_string();
    }

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
            println!("{} from {}", request.data.clone().unwrap(), stream.peer_addr().unwrap());
            chat.push(format!("{} says: {}", stream.peer_addr().unwrap() ,request.data.unwrap()))
        }
    }

    let ext : Vec<&str>= request.target.split(".").collect();

    let content = match ext.last().unwrap() {

        &"css" => {
            "text/css"
        },
        &"png" => {
          "image/png"
        }
        &"svg" => {
            "image/svg+xml"
        }
        &"ico" => {
            "image/x-icon"
        }
        _ => {
            "text/html"
        }

    };

    if request.accept.contains(&"image/png") || request.target.ends_with(".png") ||  request.target.ends_with(".ico") ||  request.target.ends_with(".svg") {

        let image = fs::read(format!(".{}", request.target)).expect(format!("Failed to read image at {}", request.target).as_str());
        let mut head = format!("HTTP/1.1 200 OK\r\nServer: Rust TCP server\r\nContent-Type: {}\r\n\r\n", content
        );
        let response = [head.as_bytes(), image.as_slice()].concat();

        stream.write_all(response.as_slice()).expect("Write failed!");
        stream.flush().unwrap()

    }
    else {
        let mut body = fs::read_to_string(format!(".{}", request.target)).expect(format!("Failed to read body at {}", request.target).as_str());

        let mut web_chat = String::new();

        for message in chat {
            web_chat = format!("{}{}<br>", web_chat,message);
        }

        body = body.replace("$CHAT", web_chat.as_str());

        let mut response = format!("HTTP/1.1 200 OK\r\nServer: Rust TCP server\r\nContent-Type: {}\r\n\r\n{}",content,body);
        stream.write_all(response.as_bytes()).expect("Write failed!");
        stream.flush().unwrap()

    }
}


fn main() -> io::Result<()> {

    let mut chat : Vec<String>= vec![];

    let mut address : String = "".to_string();
    let mut port : String = "".to_string();

    dotenv().ok();
    for (key, value) in env::vars() {
        match key.as_str() {
            "ADDRESS" => {address = value}
            "PORT" => {port = value}
            _ => {}
        }
    }

    open::that(format!("http://{}",address )).expect("Failed to open page");

    let mut listener = TcpListener::bind(format!("{}:{}",address,port));

    for stream in listener?.incoming() {
        handle_client(stream?, &mut chat)
    }


    Ok(())
}