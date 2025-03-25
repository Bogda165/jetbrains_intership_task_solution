use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::Duration,
    time::sleep,
};

pub mod http_messages;
pub mod serealize;

use serealize::{Deserialize, Serialize};

use http_messages::{
    message::HttpMessage,
    path::Path,
    request::{HttpRequest, HttpRequestMethod},
    response::HttpResponse,
};

static SERVER_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

    request.add_header("Host", "127.0.0.1:8080");
    request.add_header("User-Agent", "Rust-Client/1.0");
    request.add_header("Connection", "close");

    let request_bytes = request.serialize();

    let mut stream = TcpStream::connect(SERVER_ADDR).await?;

    stream.write_all(&request_bytes).await?;
    println!("Request sent");

    let mut buffer = Vec::new();
    let bytes_read = stream.read_to_end(&mut buffer).await?;

    if bytes_read > 0 {
        println!("Received {} bytes", bytes_read);

        let response = HttpResponse::desrialize(buffer[..bytes_read].to_vec())?;

        //println!("{:?}", response);

        println!("body length: {}", response.body.len());
    } else {
        println!("No data received");
    }

    Ok(())
}
