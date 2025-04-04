use crate::managers::{basic_manager::BasicManager, random_manager::RandomManager};
use real_manager_wrapper::test_with_server;

mod client;
pub mod managers;
mod real_manager_wrapper;

fn get_addr(args: &Vec<String>) -> Result<String, std::io::Error> {
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

fn get_hash(args: &Vec<String>) -> Result<String, std::io::Error> {
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

fn get_manager(args: &Vec<String>) -> Result<ManagerType, std::io::Error> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let addr = get_addr(&args)?;
    let hash = get_hash(&args)?;

    let manager = get_manager(&args)?;

    let res_hash = manager.start(addr.as_str());

    assert_eq!(hash, res_hash, "hashes of the same value must be the same");

    Ok(())
}
