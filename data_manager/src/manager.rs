use errors::ManagerError;
use interval_list::{chunk::Chunk, core::IntervalList};

pub trait Manager {
    fn init(data_len: usize) -> Self;

    fn get_filled_list(&self) -> &IntervalList<usize>;

    fn get_data(&self) -> &Vec<u8>;

    fn move_data(self) -> Vec<u8>;

    fn ready(&self) -> bool {
        if let Some(ref first_node) = self.get_filled_list().head {
            if first_node.begin == 0 && first_node.end == self.get_data().len() {
                return true;
            }
        }
        false
    }

    /// request chunks based on current state of an interval list
    fn request(&self) -> Result<Vec<Chunk<usize>>, ManagerError>;

    fn receive(&mut self, chunk: Vec<u8>, chunk_bounds: (usize, usize))
    -> Result<(), ManagerError>;
}

pub mod smart_manager {
    //does not make scence becuas the server is single threaded
    // // the idea of the manager is to request always from a point to the end of the data, each request will try to get point..n, (end_of_chunk_that_starts_in_point..n)
    // struct SmartMangaer {
    //     filled_list: IntervalList<u8>,
    //     sent_chunks_starts: Vec<usize>,
    //     data: Vec<u8>,
    // }

    // impl SmartMangaer {
    //     fn new(data_len: usize) -> Self {
    //         Self {
    //             data: vec![0; data_len],
    //             sent_chunks_starts: vec![],
    //             filled_list: IntervalList::new(),
    //         }
    //     }

    //     fn ready(&self) -> bool {
    //         if let Some(ref first_node) = self.filled_list.head {
    //             if first_node.begin == 0 && first_node.end == self.data.len() as u8 {
    //                 return true;
    //             }
    //         }
    //         false
    //     }

    //     fn request(&self) -> Result<Vec<Chunk<u8>>, MaganerError> {
    //         if !self.ready() {
    //             //suppose there can not be any gaps between chunks so the size of the list is always one

    //             return Ok(if let Some(ref first_node) = self.filled_list.head {
    //                 //ask first_chunk.end..len
    //                 vec![Chunk::new(first_node.end, self.data.len() as u8).unwrap()]
    //             } else {
    //                 //ask the first_chunk 0..len
    //                 vec![Chunk::new(0, self.data.len() as u8).unwrap()]
    //             });
    //         }

    //         Err(MaganerError::TheDataIsFilled)
    //     }

    //     fn receive(&mut self, chunk: Vec<u8>, chunk_bounds: (u8, u8)) {
    //         self.data[chunk_bounds.0 as usize..chunk_bounds.1 as usize]
    //             .copy_from_slice(chunk.as_slice());
    //         self.filled_list
    //             .add_chunk(Chunk::new(chunk_bounds.0, chunk_bounds.1).unwrap())
    //             .unwrap();
    //     }
    // }
}

pub mod errors {

    #[derive(Debug)]
    pub enum ManagerError {
        TheDataIsFilled,
    }

    impl std::fmt::Display for ManagerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::TheDataIsFilled => write!(f, "the data is already filled"),
            }
        }
    }
}
