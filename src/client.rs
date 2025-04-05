use data_manager::data_holder::DataHolder;
use errors::ClientError;
use http_message::http_messages::{path::Path, request::HttpRequestMethod};
use server_communicator::*;

pub struct Client {
    sender: Sender<HttpRequest>,
    receiver: Receiver<HttpResponse>,
    // ussually http servers answer with content-range header, but as our server does not do it, I will need this field(
    last_chunk_start_point: usize,
    data_len: usize,
    addr: String,
}

impl Client {
    fn get_server_addr(&self) -> &String {
        &self.addr
    }

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

    pub fn new(
        addr: &str,
        sender: Sender<HttpRequest>,
        receiver: Receiver<HttpResponse>,
    ) -> Result<Self, ServerCommunicatorError> {
        let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

        request.add_header("Host", addr);
        request.add_header("User-Agent", "Rust-Client/1.0");
        request.add_header("Connection", "close");

        sender.send(request)?;

        let response = receiver.recv_timeout(std::time::Duration::from_secs(10))?;
        if let Some(length) = response.headers.get(&"Content-Length".into()) {
            #[cfg(debug_assertions)]
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
                addr: addr.to_string(),
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

impl DataHolder for Client {
    type DataType = u8;

    type DataContainer = Vec<u8>;

    type E = ClientError;

    fn request(&mut self, bounds: (usize, usize)) -> Result<(), Self::E> {
        #[cfg(debug_assertions)]
        println!("Try sending the request");
        let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

        request.add_header("Host", &self.addr);
        request.add_header("User-Agent", "Rust-Client/1.0");
        println!("Requesting bounds = {}-{}", bounds.0, bounds.1);

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
        #[cfg(debug_assertions)]
        println!("Received data with length: {}", results.1);
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

pub mod errors {
    use data_manager::data_holder::DataHolderError;

    use super::*;

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
}
