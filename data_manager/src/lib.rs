use core_crate::{chunk::Chunk, core::IntervalList};
use errors::ManagerWrapperError;
use manager::errors::ManagerError;
use manager::{Manager, basic_manager::BasicManager};
use server_simulator::{DataHolder, Server};

pub mod manager;

pub mod server_simulator;

pub mod errors {
    use server_simulator::DataHolderError;

    // TODO fix MWE, it must not depend on ManagerWrapper and Manager just DataHolder
    use super::*;
    pub enum ManagerWrapperError<M: Manager, MW: ManagerWrapper<M>> {
        ManagerError(ManagerError),
        DataHolderError(<MW::Data as DataHolder>::E),
    }

    impl<M: Manager, MW: ManagerWrapper<M>> std::fmt::Debug for ManagerWrapperError<M, MW> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ManagerWrapperError::ManagerError(manager_error) => write!(f, "{}", manager_error),
                ManagerWrapperError::DataHolderError(err) => write!(f, "{}", err),
            }
        }
    }

    impl<M: Manager, MW: ManagerWrapper<M>, E: DataHolderError> From<E> for ManagerWrapperError<M, MW>
    where
        MW::Data: DataHolder<E = E>,
    {
        fn from(value: E) -> Self {
            Self::DataHolderError(value)
        }
    }

    impl<M: Manager, MW: ManagerWrapper<M>> From<ManagerError> for ManagerWrapperError<M, MW> {
        fn from(value: ManagerError) -> Self {
            Self::ManagerError(value)
        }
    }
}

// I could played more with generics of DataType but I decided to hard-code it due to potential errors in types converstaions
pub trait ManagerWrapper<ManagerT: Manager> {
    type Data: DataHolder<DataType = u8>;

    fn get_data_holder(&self) -> &Self::Data;

    fn get_data_holder_mut(&mut self) -> &mut Self::Data;

    fn get_manager(&self) -> &ManagerT;

    fn get_manager_mut(&mut self) -> &mut ManagerT;

    fn extra_handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
    }

    fn process_request_chunks(&mut self, request_answer: Vec<Chunk<usize>>) {}

    fn send_request(&mut self) -> Result<(), ManagerWrapperError<ManagerT, Self>>
    where
        Self: Sized,
    {
        let request_answer = match self.get_manager().request() {
            Ok(_data) => _data,
            Err(err) => {
                match err {
                    ManagerError::TheDataIsFilled => println!("Finished"),
                    _ => return Err(err.into()),
                }
                return Ok(());
            }
        };

        request_answer
            .iter()
            .try_for_each(|chunk| self.get_data_holder_mut().request((chunk.begin, chunk.end)))
            .map_err(|err| Into::<ManagerWrapperError<ManagerT, Self>>::into(err))?;

        self.process_request_chunks(request_answer);
        Ok(())
    }

    fn handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) -> Result<(), ManagerWrapperError<ManagerT, Self>>
    where
        Self: Sized,
    {
        self.get_manager_mut()
            .receive(data.clone(), requested_bounds)?;

        self.extra_handle_response(data, requested_bounds);

        Ok(())
    }

    fn start(self) -> Result<(), ManagerWrapperError<ManagerT, Self>>
    where
        Self: Sized;
}

struct TestManagerWrapper<ManagerT: Manager> {
    server: Server,
    mangaer: ManagerT,
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
        while let Some((data, (left_bound, _))) = self.get_data_holder_mut().get_response().unwrap()
        {
            let response_len = data.len();
            self.handle_response(data, (left_bound, left_bound + response_len));
        }
    }

    fn extra_handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
        self.send_request();
    }

    fn start(mut self) -> Result<(), ManagerWrapperError<ManagerT, Self>> {
        self.send_request()
    }
}

#[test]
pub fn test_server() {
    for _ in 0..100 {
        let server = Server::init_with_lower_bound(50);

        let mut tm = TestManagerWrapper {
            mangaer: BasicManager::new(server.get_len() as usize),
            server,
        };

        tm.send_request();

        let dl = tm.server.get_len();

        println!("Data len: {}", dl);
        println!("Server data: {:?}", tm.server.data);
        println!("Recieved data: {:?}", tm.mangaer.data);

        assert_eq!(
            tm.server.data,
            tm.mangaer
                .get_data()
                .into_iter()
                .map(|val| { *val })
                .collect::<Vec<u8>>()
        );
    }
}
