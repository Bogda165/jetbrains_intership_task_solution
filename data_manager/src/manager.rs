use core_crate::chunk::Chunk;
use errors::ManagerError;

use crate::IntervalList;

pub trait Manager {
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
                _ => unreachable!(),
            }
        }
    }
}

pub mod basic_manager {

    use super::*;

    pub struct BasicManager {
        pub filled_list: IntervalList<usize>,
        pub data: Vec<u8>,
    }

    /// the manager should be wait free, each time it receive any new data
    impl BasicManager {
        pub fn new(data_len: usize) -> Self {
            Self {
                data: vec![0; data_len],
                filled_list: IntervalList::new(),
            }
        }
    }

    impl Manager for BasicManager {
        fn get_filled_list(&self) -> &IntervalList<usize> {
            &self.filled_list
        }

        fn get_data(&self) -> &Vec<u8> {
            &self.data
        }

        fn move_data(self) -> Vec<u8> {
            self.data
        }

        fn request(&self) -> Result<Vec<Chunk<usize>>, ManagerError> {
            //this one will be the simples and request just request chunks sequentially.

            if !self.ready() {
                //suppose there can not be any gaps between chunks so the size of the list is always one

                assert!(self.filled_list.len() == 1 || self.filled_list.len() == 0);

                return Ok(if let Some(ref first_node) = self.filled_list.head {
                    //ask first_chunk.end..len
                    vec![Chunk::new(first_node.end, self.data.len()).unwrap()]
                } else {
                    //ask the first_chunk 0..len
                    vec![Chunk::new(0, self.data.len()).unwrap()]
                });
            }

            Err(ManagerError::TheDataIsFilled)
        }

        fn receive(
            &mut self,
            chunk: Vec<u8>,
            chunk_bounds: (usize, usize),
        ) -> Result<(), ManagerError> {
            println!("chunk_bounds: {}-{}", chunk_bounds.0, chunk_bounds.1);
            self.data[chunk_bounds.0 as usize..chunk_bounds.1 as usize]
                .copy_from_slice(chunk.as_slice());
            self.filled_list
                .add_chunk(Chunk::new(chunk_bounds.0, chunk_bounds.1).unwrap())
                .unwrap();

            if chunk_bounds.1 == self.data.len() {
                Err(ManagerError::TheDataIsFilled)
            } else {
                Ok(())
            }
        }
    }
}

pub mod random_manager {

    use rand::Rng;

    use super::*;

    pub struct RandomManager<const MIN_INTERVAL_LEN: usize> {
        pub filled_list: IntervalList<usize>,
        pub data: Vec<u8>,
    }

    /// the manager should be wait free, each time it receive any new data
    impl<const MIN_INTERVAL_LEN: usize> RandomManager<MIN_INTERVAL_LEN> {
        pub fn new(data_len: usize) -> Self {
            Self {
                data: vec![0; data_len],
                filled_list: IntervalList::new(),
            }
        }
    }

    impl<const MIN_INTERVAL_LEN: usize> Manager for RandomManager<MIN_INTERVAL_LEN> {
        fn get_filled_list(&self) -> &IntervalList<usize> {
            &self.filled_list
        }

        fn get_data(&self) -> &Vec<u8> {
            &self.data
        }

        fn move_data(self) -> Vec<u8> {
            self.data
        }

        fn request(&self) -> Result<Vec<Chunk<usize>>, ManagerError> {
            // I am goind to take all free intervals choose the random one, and request the random chunk from this interval

            if !self.ready() {
                // errors shouod never occur
                let free_intervals = self
                    .filled_list
                    .get_complement_intervals((0, self.data.len()).try_into().unwrap())
                    .unwrap();

                let free_amount = free_intervals.len();

                let mut rng = rand::rng();

                let interval = free_intervals
                    .get_interval_by_index(rng.random_range(0..free_amount))
                    .unwrap();

                let left_bound = interval.begin + MIN_INTERVAL_LEN;

                let request_chunk = if left_bound >= interval.end {
                    interval.clone()
                } else {
                    let right_bound = rng.random_range(left_bound + 1..=interval.end);
                    (left_bound, right_bound).try_into().unwrap()
                };

                return Ok(vec![request_chunk]);
            }

            Err(ManagerError::TheDataIsFilled)
        }

        fn receive(
            &mut self,
            chunk: Vec<u8>,
            chunk_bounds: (usize, usize),
        ) -> Result<(), ManagerError> {
            assert!(chunk.len() > 0);

            println!("chunk_bounds: {}-{}", chunk_bounds.0, chunk_bounds.1);
            self.data[chunk_bounds.0 as usize..chunk_bounds.1 as usize]
                .copy_from_slice(chunk.as_slice());
            self.filled_list
                .add_chunk(Chunk::new(chunk_bounds.0, chunk_bounds.1).unwrap())
                .unwrap();

            if self
                .filled_list
                .get_complement_intervals(Chunk::new(0, self.get_data().len()).unwrap())
                .unwrap()
                .len()
                == 0
            {
                Err(ManagerError::TheDataIsFilled)
            } else {
                Ok(())
            }
        }
    }
}
