use std::io::Write;
use std::net::TcpStream;
use std::path::Display;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::Thread;

use data_manager::errors::ManagerWrapperError;
use data_manager::manager::Manager;
use data_manager::manager::basic_manager::BasicManager;
use data_manager::manager::errors::ManagerError;
use data_manager::server_simulator::{DataHolder, ServerError};

use data_manager::ManagerWrapper;
use http_message::http_messages::path::Path;
use http_message::http_messages::request::{HttpRequest, HttpRequestMethod};
use http_message::http_messages::response::{self, HttpResponse};
use http_message::serialize::Serialize;
use server_communicator::{ServerCommunicator, ServerCommunicatorError};

struct Client {
    sender: Sender<HttpRequest>,
    receiver: Receiver<HttpResponse>,
    // ussually http servers answer with content-range header, but as our server does not do it, I will need this field(
    last_chunk_start_point: usize,
    data_len: usize,
}

impl Client {
    fn check_response(response: HttpResponse) -> Result<(Vec<u8>, usize), ServerCommunicatorError> {
        if response.result != 206 && response.result != 200 {
            return Err(ServerCommunicatorError::SerializeError(format!(
                "the reponse code must be 206(or 200 FULL Content): {}-{}",
                response.result_string, response.result
            )));
        }

        let len = response.body.len();
        if len == 0 {
            return Err(ServerCommunicatorError::SerializeError(
                "The length of body is 0".to_string(),
            ));
        }

        Ok((response.body, len))
    }

    fn new(
        sender: Sender<HttpRequest>,
        receiver: Receiver<HttpResponse>,
    ) -> Result<Self, ServerCommunicatorError> {
        let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

        request.add_header("Host", "127.0.0.1:8080");
        request.add_header("User-Agent", "Rust-Client/1.0");
        request.add_header("Connection", "close");

        sender.send(request)?;

        let response = receiver.recv_timeout(std::time::Duration::from_secs(10))?;
        if let Some(length) = response.headers.get(&"Content-Length".into()) {
            println!("Parsing: {}", length.value);
            let len = length
                .value
                .chars()
                .filter(|char| char.is_digit(10))
                .collect::<String>()
                .parse::<usize>()
                .map_err(|err| {
                    ServerCommunicatorError::SerializeError(format!(
                        "Error while trying to parse content of content-length header: {}",
                        err
                    ))
                })?;

            Ok(Self {
                sender,
                receiver,
                last_chunk_start_point: 0,
                data_len: len,
            })
        } else {
            Err(ServerCommunicatorError::SerializeError(
                "The server does not specify content-length".to_string(),
            ))
        }
    }
}

use data_manager::server_simulator::DataHolderError;
use sha2::Digest;

#[derive(Debug)]
pub enum ClientError {
    ServerError(ServerCommunicatorError),
}

impl From<ServerCommunicatorError> for ClientError {
    fn from(value: ServerCommunicatorError) -> Self {
        Self::ServerError(value)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::ServerError(server_communicator_error) => {
                write!(f, "{}", server_communicator_error)
            }
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl DataHolderError for ClientError {}

impl DataHolder for Client {
    type DataType = u8;

    type DataContainer = Vec<u8>;

    type E = ClientError;

    fn request(&mut self, bounds: (usize, usize)) -> Result<(), Self::E> {
        #[cfg(test)]
        println!("Try sending the request");
        let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

        request.add_header("Host", "127.0.0.1:8080");
        request.add_header("User-Agent", "Rust-Client/1.0");
        println!("requesting bounds = {}-{}", bounds.0, bounds.1);

        request.add_header("Range", &format!("bytes={}-{}", bounds.0, bounds.1));

        self.last_chunk_start_point = bounds.0;

        self.sender
            .send(request)
            .map_err(|err| Into::<ServerCommunicatorError>::into(err).into())
    }

    fn get_response(&mut self) -> Result<Option<(Self::DataContainer, (usize, usize))>, Self::E> {
        let response = self
            .receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|err| Into::<ServerCommunicatorError>::into(err))?;

        let results = Self::check_response(response)?;
        println!("HUI: received data with length: {}", results.1);
        let bounds = (
            self.last_chunk_start_point,
            self.last_chunk_start_point + results.1,
        );
        self.last_chunk_start_point += results.1;

        Ok(Some((results.0, bounds)))
    }

    fn get_data_len(&self) -> usize {
        self.data_len
    }
}

struct BasicManagerWrapper<ManagerT: Manager> {
    server: Client,
    manager: ManagerT,
}

impl<M: Manager> BasicManagerWrapper<M> {}

impl<ManagerT: Manager> ManagerWrapper<ManagerT> for BasicManagerWrapper<ManagerT> {
    type Data = Client;

    fn get_data_holder(&self) -> &Self::Data {
        &self.server
    }

    fn get_data_holder_mut(&mut self) -> &mut Self::Data {
        &mut self.server
    }

    fn get_manager(&self) -> &ManagerT {
        &self.manager
    }

    fn get_manager_mut(&mut self) -> &mut ManagerT {
        &mut self.manager
    }

    fn extra_handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
        self.send_request().unwrap()
    }

    fn start(mut self) -> Result<Vec<u8>, ManagerWrapperError<ManagerT, Self>> {
        self.send_request()?;
        let res = || -> Result<Vec<u8>, ManagerWrapperError<ManagerT, Self>> {
            while let Some(resp) = self.server.get_response()? {
                self.handle_response(resp.0, resp.1)?;
            }
            unreachable!()
            //Ok(self.manager.move_data())
        }();

        if let Err(ManagerWrapperError::ManagerError(ManagerError::TheDataIsFilled)) = res {
            println!("Finished");
            Ok(self.manager.move_data())
        } else {
            res
        }
    }
}

use sha2::Sha256;

pub mod my_hex {
    #[derive(Debug, Clone)]
    pub enum HexError {
        OutOfBounds,
    }

    pub trait ToHex {
        fn to_hex(val: u8) -> Result<Self, HexError>
        where
            Self: Sized;

        fn encode<I: Iterator<Item = u8>, R: FromIterator<Self>>(data: I) -> R
        where
            Self: Sized;
    }

    impl ToHex for char {
        fn to_hex(val: u8) -> Result<Self, HexError> {
            Ok(match val {
                0..=9 => (b'0' + val) as char,
                10..16 => (b'a' + val - 10) as char,
                _ => return Err(HexError::OutOfBounds),
            })
        }

        fn encode<I: Iterator<Item = u8>, R: FromIterator<Self>>(data: I) -> R {
            data.into_iter()
                .flat_map(|byte| {
                    let high = byte >> 4;
                    let low = byte & 0x0F;
                    [Self::to_hex(high).unwrap(), Self::to_hex(low).unwrap()]
                })
                .collect()
        }
    }

    mod tests {
        use super::*;

        #[test]
        fn test_single_byte() {
            let data = [0x4F];
            let hex: String = char::encode(data.iter().copied());
            assert_eq!(hex, "4f");
        }

        #[test]
        fn test_multiple_bytes() {
            let data = [0xAB, 0xCD, 0xEF];
            let hex: String = char::encode(data.iter().copied());
            assert_eq!(hex, "abcdef");
        }

        #[test]
        fn test_empty() {
            let data: [u8; 0] = [];
            let hex: String = char::encode(data.iter().copied());
            assert_eq!(hex, "");
        }

        #[test]
        fn test_all_nibbles() {
            let data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
            let hex: String = char::encode(data.iter().copied());
            assert_eq!(hex, "0123456789abcdef");
        }
    }
}

use my_hex::ToHex;

fn hash_to_string(data: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&data);

    let res = hasher.finalize();

    char::encode(res.into_iter())
}

use data_manager::{TestManagerWrapper, server_simulator::Server};

pub fn test_with_server<M: Manager>(server_addr: &str) -> String {
    let (sc, (r, s)) = ServerCommunicator::new(server_addr).unwrap();
    sc.start();
    let client = Client::new(s, r).unwrap();
    let data_len = client.data_len;
    let bm = BasicManagerWrapper {
        server: client,
        manager: M::init(data_len),
    };

    //hash
    let res = bm.start().unwrap();

    hash_to_string(res)
}

fn test_simulation<M: Manager>() -> String {
    let server = Server::init_with_lower_bound(50);

    let mut tm = TestManagerWrapper {
        mangaer: M::init(server.get_data_len()),
        server,
    };

    tm.send_request();

    let dl = tm.server.get_len();

    println!("Data len: {}", dl);
    println!("Server data: {:?}", tm.server.data);
    println!("Recieved data: {:?}", tm.mangaer.get_data());

    assert_eq!(
        tm.server.data,
        tm.mangaer
            .get_data()
            .into_iter()
            .map(|val| { *val })
            .collect::<Vec<u8>>()
    );

    hash_to_string(tm.mangaer.move_data())
}

#[cfg(test)]
pub mod tests {

    use data_manager::manager::random_manager::RandomManager;

    use super::*;

    #[test]
    fn test_prod_basic() {
        println!(
            "Hash: {}",
            test_with_server::<BasicManager>("127.0.0.1:8080")
        );
    }

    #[test]
    fn test_prod_radnom() {
        println!(
            "Hash: {}",
            test_with_server::<RandomManager>("127.0.0.1:8080")
        );
    }

    #[test]
    fn test_basic() {
        test_simulation::<BasicManager>();
    }

    #[test]
    fn test_radnom() {
        test_simulation::<RandomManager>();
    }
}
