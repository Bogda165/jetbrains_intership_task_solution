use crate::serialize::Serialize;

pub mod path {
    #[derive(Debug, Clone)]
    pub struct Path {
        pub path: String,
    }

    impl Path {
        pub fn new(path: String) -> Result<Self, ()> {
            Ok(Self {
                path: Self::create_with_cheking(path)?,
            })
        }

        fn create_with_cheking(path: String) -> Result<String, ()> {
            //TODO: check the path
            Ok(path)
        }
    }

    impl Default for Path {
        fn default() -> Self {
            Self {
                path: "/".to_string(),
            }
        }
    }
}

pub mod header {
    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    pub struct HeaderName {
        pub name: String,
    }

    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    pub struct HeaderValue {
        pub value: String,
    }
}

pub mod message {
    use super::header::{HeaderName, HeaderValue};
    use std::collections::HashMap;

    pub trait HttpMessage {
        fn get_start_line(&self) -> String;
        fn get_headers(&self) -> &HashMap<HeaderName, HeaderValue>;
        fn get_body(&self) -> Vec<u8>;
    }
}

pub mod response {
    use crate::serialize::Deserialize;

    use super::*;
    use header::{HeaderName, HeaderValue};
    use message::HttpMessage;
    use std::{collections::HashMap, fmt::Display};

    #[derive(Debug)]
    pub struct HttpResponse {
        pub protocol: String,
        pub result: u16,
        pub result_string: String,
        pub headers: HashMap<HeaderName, HeaderValue>,
        pub body: Vec<u8>,
    }

    impl Display for HttpResponse {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{} {} {}\n Headers:\n {:?}\n data length: {}",
                self.protocol,
                self.result,
                self.result_string,
                self.headers,
                self.body.len()
            )
        }
    }

    impl HttpResponse {
        pub fn new(result: u16, result_string: &str, protocol: &str) -> Self {
            Self {
                result,
                result_string: result_string.to_string(),
                protocol: protocol.to_string(),
                headers: HashMap::new(),
                body: vec![],
            }
        }

        pub fn add_header(&mut self, name: &str, value: &str) {
            self.headers.insert(
                HeaderName {
                    name: name.to_string(),
                },
                HeaderValue {
                    value: value.to_string(),
                },
            );
        }
    }

    impl HttpMessage for HttpResponse {
        fn get_start_line(&self) -> String {
            format!("{:?} {} {}", self.protocol, self.result, self.result_string,)
        }

        fn get_headers(&self) -> &HashMap<HeaderName, HeaderValue> {
            &self.headers
        }

        fn get_body(&self) -> Vec<u8> {
            self.body.clone()
        }
    }

    impl Deserialize for HttpResponse {
        fn desrialize(buffer: Vec<u8>) -> Result<Self, String>
        where
            Self: Sized,
        {
            //divide by lines
            let mut current_begin_index = 0;
            let mut lines = buffer.split(|&b| b == b'\n').map(|line| {
                let line_len = line.len() + 1;
                current_begin_index += line_len;
                if !line.is_empty() && line[line.len() - 1] == b'\r' {
                    (&line[..line.len() - 1], current_begin_index - line_len)
                } else {
                    (line, current_begin_index - line_len)
                }
            });

            // parse first line
            let (protocol, result, result_string) = {
                if let Some((first_line, _)) = lines.next() {
                    let mut elements = first_line.split(|&b| b == b' ');

                    // the string will always have at least first element
                    let protocol = std::str::from_utf8(elements.next().unwrap())
                        .map_err(|err| "utf8 error while parsing (element 1)")?;

                    let result = std::str::from_utf8(if let Some(val) = elements.next() {
                        Ok(val)
                    } else {
                        Err("there is only one element in first string")
                    }?)
                    .map_err(|err| "utf8 error while parsing(element 2)")?
                    .parse::<u16>()
                    .map_err(|err| format!("{:?}", err))?;

                    let result_string = std::str::from_utf8(if let Some(val) = elements.next() {
                        Ok(val)
                    } else {
                        Err("there is only one element in first string")
                    }?)
                    .map_err(|err| "utf8 error while parsing(element 2)")?;

                    if !elements.next().is_none() {
                        // Err(format!(
                        //     "to match arguments in {:?}",
                        //     (protocol, result, result_string)
                        // ))
                        Ok((protocol, result, result_string))
                    } else {
                        Ok((protocol, result, result_string))
                    }
                } else {
                    Err("there is not first line".to_string())
                }
            }?;

            let headers = {
                let mut map = HashMap::new();
                let mut header_amount = 0;
                while let Some((line, _)) = lines.next() {
                    if std::str::from_utf8(line)
                        .map_err(|err| format!("{:?}", err))?
                        .is_empty()
                    {
                        println!("header amount: {}", header_amount);
                        break;
                    }

                    let mut elements = line.split(|&b| b == b':');

                    let header: Result<(&str, &str), String> = {
                        let name = std::str::from_utf8(if let Some(val) = elements.next() {
                            Ok(val)
                        } else {
                            Err("error while parsing headers name".to_string())
                        }?)
                        .map_err(|_err| "utf8 error while parsing(element 2)".to_string())?;

                        let value = std::str::from_utf8(if let Some(val) = elements.next() {
                            Ok(val)
                        } else {
                            Err("error while parsing headers value".to_string())
                        }?)
                        .map_err(|_err| "utf8 error while parsing(element 2)".to_string())?;

                        // if let Some(next_element) = elements.next() {
                        //     Err(format!(
                        //         "too many elements in header. Element: {:?}",
                        //         next_element
                        //     ))
                        // } else {
                        //     Ok((name, value))
                        // }
                        Ok((name, value))
                    };
                    let header = header?;

                    println!("Header: {}:{}", header.0, header.1);

                    map.insert(
                        HeaderName {
                            name: header.0.to_string(),
                        },
                        HeaderValue {
                            value: header.1.to_string(),
                        },
                    );
                    header_amount += 1;
                }
                Result::<HashMap<HeaderName, HeaderValue>, String>::Ok(map)
            }?;

            let body = if let Some(first_body_line) = lines.next() {
                Ok(buffer[first_body_line.1..].to_vec())
            } else {
                Err("there is not body".to_string())
            }?;

            // let body = lines.try_fold(Vec::new(), |mut vec, (line, _)| {
            //     vec.extend(line);
            //     Result::<Vec<u8>, String>::Ok(vec)
            // })?;

            Ok({
                let mut response = HttpResponse::new(result, result_string, protocol);

                response.headers = headers;

                response.body = body;

                response
            })
        }
    }
}

pub mod request {
    use std::collections::HashMap;

    use crate::serialize::Serialize;

    use super::*;
    use header::{HeaderName, HeaderValue};
    use message::HttpMessage;
    use path::Path;
    use response::HttpResponse;
    #[derive(Debug, Clone)]
    pub enum HttpRequestMethod {
        GET,
        POST,
        UPDATE,
        // and the etc.
    }

    #[derive(Debug, Clone)]
    pub struct HttpRequest {
        pub method: HttpRequestMethod,
        pub request_target: Path,
        pub protocol: String,
        pub headers: HashMap<HeaderName, HeaderValue>,
        pub body: Vec<u8>,
    }

    impl HttpRequest {
        pub fn new(method: HttpRequestMethod, request_target: Path, protocol: &str) -> Self {
            Self {
                method,
                request_target,
                protocol: protocol.to_string(),
                headers: HashMap::new(),
                body: vec![],
            }
        }

        pub fn add_header(&mut self, name: &str, value: &str) {
            self.headers.insert(
                HeaderName {
                    name: name.to_string(),
                },
                HeaderValue {
                    value: value.to_string(),
                },
            );
        }
    }

    impl HttpMessage for HttpRequest {
        fn get_start_line(&self) -> String {
            format!(
                "{:?} {} {}",
                self.method, self.request_target.path, self.protocol
            )
        }

        fn get_headers(&self) -> &HashMap<HeaderName, HeaderValue> {
            &self.headers
        }

        fn get_body(&self) -> Vec<u8> {
            self.body.clone()
        }
    }

    impl<T: HttpMessage> Serialize for T {
        fn serialize(self) -> Vec<u8> {
            let mut result = vec![];
            // ading a first line
            result.extend(format!("{}\r\n", self.get_start_line()).as_bytes());

            // adding headers
            self.get_headers().into_iter().for_each(|(hn, hv)| {
                result.extend(format!("{}: {}\r\n", hn.name, hv.value).as_bytes());
            });

            //adding emtpty line

            result.extend(b"\r\n");

            //adding body

            result.extend(self.get_body());

            result
        }
    }

    impl Default for HttpRequest {
        fn default() -> Self {
            HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1")
        }
    }

    #[test]
    fn test1() {
        let mut request = HttpRequest::new(HttpRequestMethod::GET, Path::default(), "HTTP/1.1");

        request.add_header("Host", "127.0.0.1:8080");
        request.add_header("User-Agent", "Rust-Client/1.0");
        request.add_header("Connection", "close");

        println!("{:?}", request.serialize());
    }
}
