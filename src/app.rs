use crate::managers::{basic_manager::BasicManager, random_manager::RandomManager};
use crate::real_manager_wrapper::test_with_server;

mod arguments {
    use super::ManagerType;

    pub fn get_addr(args: &Vec<String>) -> Result<String, std::io::Error> {
        let mut iterator = args.iter();

        while let Some(val) = iterator.next() {
            if val.as_str() == "--addr" {
                break;
            }
        }

        if let Some(addr) = iterator.next() {
            Ok(addr.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The run script must include server address",
            ))
        }
    }

    pub fn get_hash(args: &Vec<String>) -> Result<String, std::io::Error> {
        let mut iterator = args.iter();

        while let Some(val) = iterator.next() {
            if val.as_str() == "--hash" {
                break;
            }
        }

        if let Some(hash) = iterator.next() {
            Ok(hash.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The run script must include hash of the data",
            ))
        }
    }

    pub fn get_manager(args: &Vec<String>) -> Result<ManagerType, std::io::Error> {
        let mut iterator = args.iter();

        while let Some(val) = iterator.next() {
            if val.as_str() == "--manager" {
                break;
            }
        }

        if let Some(manager) = iterator.next() {
            match manager.as_str() {
                "basic_manager" => Ok(ManagerType::BasicManager),
                "random_manager" => Ok(ManagerType::RandomManager),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "no such manager",
                )),
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The run script must include server address",
            ))
        }
    }
}

#[derive(Debug)]
pub enum ManagerType {
    BasicManager,
    RandomManager,
}

impl ManagerType {
    fn start(self, addr: &str) -> String {
        match self {
            ManagerType::BasicManager => test_with_server::<BasicManager>(addr),
            ManagerType::RandomManager => test_with_server::<RandomManager>(addr),
        }
    }
}

#[derive(Debug)]
pub struct Application {
    server_addr: String,
    manager_type: ManagerType,
    hash: String,
}

impl Application {
    fn try_get_arg(key: (&str, &str)) -> Result<String, std::io::Error> {
        let try_arg: Vec<String> = std::env::args().collect();

        let mut iter = try_arg.into_iter();

        if let Some(_) = iter.find(|arg| arg == key.0) {
            return match iter.next() {
                Some(val) => Ok(val),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "incorrect argument",
                )),
            };
        }

        std::env::var(key.1).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "{}, {} wasn't find neither in script arguments, or evn variable",
                    key.0, key.1
                ),
            )
        })
    }

    pub fn new() -> Result<Self, std::io::Error> {
        let addr = Self::try_get_arg(("--addr", "ADDR"));
        let manager = Self::try_get_arg(("--manager", "MANAGER"));

        let manager = if let Ok(manager) = manager {
            match &*manager {
                "basic_manager" => Ok(ManagerType::BasicManager),
                "random_manager" => Ok(ManagerType::RandomManager),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("no such manager {}", manager),
                )),
            }
        } else {
            Err(manager.err().unwrap())
        };

        let hash = Self::try_get_arg(("--hash", "HASH"));

        Ok(Self {
            server_addr: addr?,
            manager_type: manager?,
            hash: hash?,
        })
    }

    pub fn start(self) -> Result<(), std::io::Error> {
        let res = self.manager_type.start(&self.server_addr);

        if res == self.hash {
            println!("Hashes are the same");
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Hash client constructed: {}, provided hash: {}",
                    res, self.hash
                ),
            ))
        }
    }
}
