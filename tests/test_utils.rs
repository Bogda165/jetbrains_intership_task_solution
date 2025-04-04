pub mod server {
    use std::{error::Error, fmt::Display};

    use errors::ServerError;
    pub use rand::{TryRngCore, rngs::OsRng};

    pub use data_manager::data_holder::{DataHolder, DataHolderError};

    pub mod errors {
        use super::*;

        #[derive(Debug)]
        pub enum ServerError {
            OutOfBounds((u8, u8), u8),
        }

        impl Display for ServerError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    ServerError::OutOfBounds(bounds, length) => {
                        write!(
                            f,
                            "Bounds: {}->{} are not valid for array of size length {}",
                            bounds.0, bounds.1, length
                        )
                    }
                    _ => unreachable!(),
                }
            }
        }

        impl Error for ServerError {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }

            fn description(&self) -> &str {
                "description() is deprecated; use Display"
            }

            fn cause(&self) -> Option<&dyn Error> {
                self.source()
            }

            //fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {}
        }

        impl DataHolderError for ServerError {}
    }

    impl DataHolder for Server {
        type DataType = u8;
        type DataContainer = Vec<u8>;
        type E = ServerError;

        fn request(&mut self, bounds: (usize, usize)) -> Result<(), ServerError> {
            let response = self
                .get_data_from_range((bounds.0 as u8, bounds.1 as u8))
                .map(Vec::<u8>::from)?;

            self.records.push((response, (bounds.0, bounds.1)));

            Ok(())
        }

        fn get_response(
            &mut self,
        ) -> Result<Option<(Self::DataContainer, (usize, usize))>, Self::E> {
            Ok(self.records.pop())
        }

        fn get_data_len(&self) -> usize {
            self.get_len() as usize
        }
    }

    #[derive(Clone, Debug)]
    pub struct Server {
        pub data: Vec<u8>,
        pub records: Vec<(Vec<u8>, (usize, usize))>,
    }

    impl Server {
        pub fn new() -> Self {
            Self::init(|| OsRng.try_next_u32().unwrap() as u8)
        }

        pub fn init<F: FnOnce() -> u8>(len_fn: F) -> Self {
            let mut raw_data = vec![0_u8; len_fn() as usize];
            OsRng.try_fill_bytes(raw_data.as_mut_slice()).unwrap();

            Self {
                data: raw_data,
                records: vec![],
            }
        }

        pub fn init_with_lower_bound(lb: u8) -> Self {
            Self::init(|| ((OsRng.try_next_u32().unwrap() as u8).saturating_sub(lb) + lb) as u8)
        }

        pub fn get_len(&self) -> u8 {
            self.data.len() as u8
        }

        pub fn get_data_from_range(&self, bounds: (u8, u8)) -> Result<&[u8], ServerError> {
            println!("Get data from range: {:?}", bounds);

            assert!(bounds.0 < bounds.1);
            if bounds.1 > self.get_len() {
                return Err(ServerError::OutOfBounds(bounds, self.get_len()));
            }

            let right_bound =
                ((OsRng.try_next_u32().unwrap() as u8) % (bounds.1 - bounds.0) + bounds.0) as usize;

            Ok(
                &self.data[(bounds.0 as usize)..if right_bound == bounds.0 as usize {
                    bounds.0 as usize + 1
                } else {
                    right_bound
                }],
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_server_init() {
            let mut prev_data = Vec::new();

            for _ in 0..10 {
                let server = Server::new();
                assert_ne!(prev_data, server.data);

                prev_data = server.data;
            }
        }

        // #[test]
        // fn test_get_function() {
        //     let server = Server::init_with_lower_bound(50);
        //     let bounds = (10, 50);
        //     let mut prev_data: &[u8] = &[0_u8; 0];

        //     println!("data in server len: {}", server.get_len());
        //         let new_data = server.get_data_from_range(bounds).unwrap();
        //         assert_ne!(new_data, prev_data);
        //         prev_data = new_data;
        //     }
        // }
    }
}

pub mod test_wrapper {

    pub use data_manager::{
        data_holder::DataHolder,
        manager::{Manager, errors::ManagerError},
        manager_wrapper::{ManagerWrapper, errors::ManagerWrapperError},
    };
    use interval_list::core::Chunk;

    use super::server::*;

    pub struct TestManagerWrapper<ManagerT: Manager> {
        pub server: Server,
        pub mangaer: ManagerT,
    }

    impl<ManagerT: Manager> ManagerWrapper<ManagerT> for TestManagerWrapper<ManagerT> {
        type Data = Server;

        fn get_data_holder(&self) -> &Self::Data {
            &self.server
        }

        fn get_data_holder_mut(&mut self) -> &mut Self::Data {
            &mut self.server
        }

        fn get_manager(&self) -> &ManagerT {
            &self.mangaer
        }

        fn get_manager_mut(&mut self) -> &mut ManagerT {
            &mut self.mangaer
        }

        fn process_request_chunks(&mut self, request_answer: Vec<Chunk<usize>>) {
            // in context of sever get_response can never return an error
            while let Some((data, (left_bound, _))) =
                self.get_data_holder_mut().get_response().unwrap()
            {
                let response_len = data.len();
                if let Err(ManagerWrapperError::ManagerError(ManagerError::TheDataIsFilled)) =
                    self.handle_response(data, (left_bound, left_bound + response_len))
                {
                    println!("Finalize");
                    break;
                }
            }
        }

        fn extra_handle_response(
            &mut self,
            data: Vec<<Self::Data as DataHolder>::DataType>,
            requested_bounds: (usize, usize),
        ) {
            self.send_request();
        }

        fn start(mut self) -> Result<Vec<u8>, ManagerWrapperError<ManagerT, Self>> {
            self.send_request();

            Ok(self.mangaer.move_data())
        }
    }
}
