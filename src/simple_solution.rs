use std::io::Write;
use std::net::TcpStream;

use data_manager::manager::Manager;
use data_manager::server_simulator::{DataHolder, ServerError};

use http_message::http_messages::path::Path;
use http_message::http_messages::request::{HttpRequest, HttpRequestMethod};
use http_message::serialize::Serialize;

struct Client {
    stream: TcpStream,
    data_len: usize,
    ip: &'static str,
}

impl DataHolder for Client {
    type DataType = u8;

    type DataContainer = Vec<u8>;

    type E = std::io::Error;

    fn request(&mut self, bounds: (usize, usize)) -> Result<(), Self::E> {
        let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

        request.add_header("Host", "127.0.0.1:8080");
        request.add_header("User-Agent", "Rust-Client/1.0");
        request.add_header("Range", "bytes=0-1000");

        let request_bytes = request.serialize();

        self.stream.write_all(&*request_bytes)?;

        Ok(())
    }

    fn get_response(&mut self) -> Option<(Self::DataContainer, (usize, usize))> {
        todo!()
    }

    fn get_data_len(&self) -> usize {
        todo!()
    }
}
