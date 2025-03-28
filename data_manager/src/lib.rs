use std::path::Display;

use core_crate::core::{IntervalList, chunk::*};
use rand::Fill;
use server_simulator::{DataHolder, Server};

pub mod tests;

pub mod server_simulator;

// trait Manager {
//     type Data: DataHolder<DataType: Clone + std::fmt::Debug + PartialOrd>;

//     fn get_interval_list(&self) -> &IntervalList<u8>;

//     fn get_interval_list_mut(&mut self) -> &mut IntervalList<u8>;

//     fn addititonal_function_before_request(&self);

//     fn send_request(&mut self, data_holder: Self::Data, bounds: (usize, usize)) {
//         data_holder.request(bounds).unwrap();

//         self.addititonal_function_before_request();
//     }

//     fn handle_response(
//         &mut self,
//         data: Vec<<Self::Data as DataHolder>::DataType>,
//         requested_bounds: (
//             <Self::Data as DataHolder>::DataType,
//             <Self::Data as DataHolder>::DataType,
//         ),
//     );
// }

struct TestManager {
    server: Server,
    mangaer: MOManager,
}

impl TestManager {
    pub fn request_server(&mut self) {
        // get range
        let bounds = match self.mangaer.request() {
            Ok(bounds) => bounds,
            Err(_) => {
                println!("Finished");
                vec![]
            }
        };

        bounds.into_iter().for_each(|bounds| {
            let data = self
                .server
                .get_data_from_range(bounds.clone().into())
                .unwrap();

            self.handle_response(
                data.to_vec(),
                (bounds.begin, bounds.begin + data.len() as u8),
            );
        });
    }

    pub fn handle_response(&mut self, data: Vec<u8>, bounds: (u8, u8)) {
        self.mangaer.receive(data, bounds);
        self.request_server();
    }
}

#[test]
pub fn test_server() {
    for _ in 0..100 {
        let server = Server::init_with_lower_bound(50);

        let mut tm = TestManager {
            mangaer: MOManager::new(server.get_len() as usize),
            server,
        };

        tm.request_server();

        let dl = tm.server.get_len();

        println!("Data len: {}", dl);
        println!("Server data: {:?}", tm.server.data);
        println!("Recieved data: {:?}", tm.mangaer.data);

        assert_eq!(tm.server.data, tm.mangaer.data);
    }
}

struct SmartMangaer {
    filled_list: IntervalList<u8>,
    data: Vec<u8>,
}

impl SmartMangaer {
    fn new(data_len: usize) -> Self {
        Self {
            data: vec![0; data_len],
            filled_list: IntervalList::new(),
        }
    }

    fn ready(&self) -> bool {
        if let Some(ref first_node) = self.filled_list.head {
            if first_node.begin == 0 && first_node.end == self.data.len() as u8 {
                return true;
            }
        }
        false
    }

    fn request(&self) -> Result<Vec<Chunk<u8>>, MaganerError> {
        if !self.ready() {
            //suppose there can not be any gaps between chunks so the size of the list is always one

            return Ok(if let Some(ref first_node) = self.filled_list.head {
                //ask first_chunk.end..len
                vec![Chunk::new(first_node.end, self.data.len() as u8).unwrap()]
            } else {
                //ask the first_chunk 0..len
                vec![Chunk::new(0, self.data.len() as u8).unwrap()]
            });
        }

        Err(MaganerError::TheDataIsFilled)
    }

    fn receive(&mut self, chunk: Vec<u8>, chunk_bounds: (u8, u8)) {
        self.data[chunk_bounds.0 as usize..chunk_bounds.1 as usize]
            .copy_from_slice(chunk.as_slice());
        self.filled_list
            .add_chunk(Chunk::new(chunk_bounds.0, chunk_bounds.1).unwrap())
            .unwrap();
    }
}

struct MOManager {
    filled_list: IntervalList<u8>,
    data: Vec<u8>,
}

#[derive(Debug)]
enum MaganerError {
    TheDataIsFilled,
}

impl std::fmt::Display for MaganerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TheDataIsFilled => write!(f, "the data is already filled"),
            _ => unreachable!(),
        }
    }
}

/// the manager should be wait free, each time it receive any new data
impl MOManager {
    fn new(data_len: usize) -> Self {
        Self {
            data: vec![0; data_len],
            filled_list: IntervalList::new(),
        }
    }

    fn ready(&self) -> bool {
        if let Some(ref first_node) = self.filled_list.head {
            if first_node.begin == 0 && first_node.end == self.data.len() as u8 {
                return true;
            }
        }
        false
    }

    // request chunks based on current state of an interval list
    fn request(&self) -> Result<Vec<Chunk<u8>>, MaganerError> {
        //this one will be the simples and request just request chunks sequentially.

        if !self.ready() {
            //suppose there can not be any gaps between chunks so the size of the list is always one

            return Ok(if let Some(ref first_node) = self.filled_list.head {
                //ask first_chunk.end..len
                vec![Chunk::new(first_node.end, self.data.len() as u8).unwrap()]
            } else {
                //ask the first_chunk 0..len
                vec![Chunk::new(0, self.data.len() as u8).unwrap()]
            });
        }

        Err(MaganerError::TheDataIsFilled)
    }

    // receive the chunk and modify the list
    fn receive(&mut self, chunk: Vec<u8>, chunk_bounds: (u8, u8)) {
        self.data[chunk_bounds.0 as usize..chunk_bounds.1 as usize]
            .copy_from_slice(chunk.as_slice());
        self.filled_list
            .add_chunk(Chunk::new(chunk_bounds.0, chunk_bounds.1).unwrap())
            .unwrap();
    }
}
