use core::fmt;
use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
    sync::mpsc::{RecvError, SendError, channel},
};

pub mod simple_solution;

use std::sync::mpsc::{Receiver, Sender};

use http_message::http_messages::{
    header::HeaderName,
    path::Path,
    request::{HttpRequest, HttpRequestMethod},
    response::HttpResponse,
};

use server_communicator::{ServerCommunicator, ServerCommunicatorError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

    request.add_header("Host", "127.0.0.1:8080");
    request.add_header("User-Agent", "Rust-Client/1.0");
    request.add_header("Connection", "close");
    request.add_header("Range", "bytes=1-1000");

    let (communicator, (receiver, sender)) = ServerCommunicator::new("127.0.0.1:8080").unwrap();
    communicator.start();

    let _sender = sender.clone();
    std::thread::scope(move |f| {
        f.spawn(move || {
            for _ in 0..11 {
                let response = receiver
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap();

                println!("{}", response);
            }
        });

        f.spawn(move || {
            for _ in 0..10 {
                _sender.send(request.clone()).unwrap();
            }
            println!("all requests have been sent");
        });
    });

    let mut terminate_request =
        HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

    terminate_request.add_header("X-Force-Terminate", "");

    sender.send(terminate_request.clone()).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(5));
    sender.send(terminate_request.clone()).unwrap();

    Ok(())
}
