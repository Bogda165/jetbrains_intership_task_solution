use core_crate::{chunk::Chunk, core::IntervalList};
use manager::{Manager, basic_manager::BasicManager};
use server_simulator::{DataHolder, Server};

mod tests;

pub mod manager;

pub mod server_simulator;

// I could played more with generics of DataType but I decided to hard-code it due to potential errors in types converstaions
trait ManagerWrapper<ManagerT: Manager> {
    type Data: DataHolder<DataType = u8>;

    fn get_data_holder(&self) -> &Self::Data;

    fn get_manager(&self) -> &ManagerT;

    fn get_manager_mut(&mut self) -> &mut ManagerT;

    fn extra_handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
    }

    fn process_request_data(&mut self, request_answer: Vec<Chunk<usize>>) {}

    fn send_request(&mut self) {
        let request_answer = if let Ok(_data) = self.get_manager().request() {
            _data
        } else {
            println!("Finished");
            return;
        };

        self.process_request_data(request_answer);
    }

    fn handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
        self.get_manager_mut()
            .receive(data.clone(), requested_bounds);

        self.extra_handle_response(data, requested_bounds);
    }
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

    fn get_manager(&self) -> &ManagerT {
        &self.mangaer
    }

    fn get_manager_mut(&mut self) -> &mut ManagerT {
        &mut self.mangaer
    }

    fn process_request_data(&mut self, request_answer: Vec<Chunk<usize>>) {
        request_answer.into_iter().for_each(|bounds| {
            let data = self
                .server
                .get_data_from_range(bounds.clone().convert::<u8>().unwrap().into())
                .unwrap();

            self.handle_response(data.to_vec(), (bounds.begin, bounds.begin + data.len()));
        });
    }

    fn extra_handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
        self.send_request();
    }
}

impl<ManagerT: Manager> TestManagerWrapper<ManagerT> {
    pub fn _request_server(&mut self) {
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
                .get_data_from_range(bounds.clone().convert::<u8>().unwrap().into())
                .unwrap();

            self._handle_response(data.to_vec(), (bounds.begin, bounds.begin + data.len()));
        });
    }

    pub fn _handle_response(&mut self, data: Vec<u8>, bounds: (usize, usize)) {
        self.mangaer.receive(data, (bounds.0, bounds.1));
        self._request_server();
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
                .data
                .into_iter()
                .map(|val| { val as u8 })
                .collect::<Vec<u8>>()
        );
    }
}
