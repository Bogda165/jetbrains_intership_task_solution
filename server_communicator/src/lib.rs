use http_message::{
    http_messages::{header::HeaderName, request::HttpRequest, response::HttpResponse},
    serialize::*,
};
use std::{
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
    sync::mpsc::{Receiver, RecvError, SendError, Sender, channel},
};

/// Abstraction for communication with server. When the server will be updated to later http version, or will allow connection keep alive header. The Comunicator need to be improved.
pub struct ServerCommunicator {
    requests: Receiver<HttpRequest>,
    respons: Sender<HttpResponse>,
}

pub enum ServerCommunicatorError {
    NoHostNameinTheHeader,
    TcpError(std::io::Error),
    SerializeError(String),
    ChannelError(String),
}

impl From<std::io::Error> for ServerCommunicatorError {
    fn from(value: std::io::Error) -> Self {
        Self::TcpError(value)
    }
}

impl<T> From<SendError<T>> for ServerCommunicatorError {
    fn from(value: SendError<T>) -> Self {
        Self::ChannelError(format!("{}", value))
    }
}

impl From<RecvError> for ServerCommunicatorError {
    fn from(value: RecvError) -> Self {
        Self::ChannelError(format!("{}", value))
    }
}

impl std::fmt::Display for ServerCommunicatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHostNameinTheHeader => write!(f, "Host header wasnt find in the request"),
            Self::TcpError(err) => write!(
                f,
                "Attemp to connect to tcp socket finished with error: {}",
                err
            ),
            Self::SerializeError(msg) => write!(f, "Serialize error with message: {}", msg),
            Self::ChannelError(msg) => write!(f, "Channler error {}", msg),

            _ => unreachable!(),
        }
    }
}

impl ServerCommunicator {
    pub fn new(
        ip: &str,
    ) -> Result<(Self, (Receiver<HttpResponse>, Sender<HttpRequest>)), std::io::Error> {
        //create both chanels
        let (tx_request, rx_request): (Sender<HttpRequest>, Receiver<HttpRequest>) = channel();
        let (tx_response, rx_response): (Sender<HttpResponse>, Receiver<HttpResponse>) = channel();

        Ok((
            Self {
                requests: rx_request,
                respons: tx_response,
            },
            (rx_response, tx_request),
        ))
    }

    fn workflow(&mut self, request: HttpRequest) -> Result<(), ServerCommunicatorError> {
        let addr = &request
            .headers
            .get(&HeaderName {
                name: "Host".to_string(),
            })
            .ok_or_else(|| ServerCommunicatorError::NoHostNameinTheHeader)?
            .value;

        let mut stream = TcpStream::connect(addr)?;

        stream.write_all(&*request.serialize())?;

        let mut buffer = vec![];

        let response_len = stream.read_to_end(&mut buffer)?;

        if response_len == 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "The lenght of read data is 0",
            ))?
        }

        let response = HttpResponse::desrialize(buffer)
            .map_err(|msg| ServerCommunicatorError::SerializeError(msg))?;

        self.respons.send(response)?;

        Ok(())
    }

    /// Incredibly simple version of communication, because server terminates connection after the request, I create new connection for each request.
    pub fn start(mut self) {
        std::thread::spawn(move || {
            while let Ok(request) = self.requests.recv() {
                match self.workflow(request) {
                    Ok(_) => println!("The value was send through the channel"),
                    Err(err) => eprintln!("{}", err),
                }
            }
        });
    }
}
