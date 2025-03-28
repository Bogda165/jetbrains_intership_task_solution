use std::{io::Write, net::TcpStream};

use crate::{
    HttpRequest,
    http_messages::{request::HttpRequestMethod, response},
    serealize::Serialize,
};

pub struct Data {
    len: u32,
    data: Vec<u8>,
}
