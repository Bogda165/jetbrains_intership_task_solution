use crate::client::*;
use data_manager::{
    data_holder::DataHolder,
    manager::Manager,
    manager::errors::ManagerError,
    manager_wrapper::{ManagerWrapper, errors::ManagerWrapperError},
};

use server_communicator::*;
use sha2::Digest;

struct RealManagerWrapper<ManagerT: Manager> {
    server: Client,
    manager: ManagerT,
}

impl<M: Manager> RealManagerWrapper<M> {}

impl<ManagerT: Manager> ManagerWrapper<ManagerT> for RealManagerWrapper<ManagerT> {
    type Data = Client;

    fn get_data_holder(&self) -> &Self::Data {
        &self.server
    }

    fn get_data_holder_mut(&mut self) -> &mut Self::Data {
        &mut self.server
    }

    fn get_manager(&self) -> &ManagerT {
        &self.manager
    }

    fn get_manager_mut(&mut self) -> &mut ManagerT {
        &mut self.manager
    }

    fn extra_handle_response(
        &mut self,
        data: Vec<<Self::Data as DataHolder>::DataType>,
        requested_bounds: (usize, usize),
    ) {
        self.send_request().unwrap()
    }

    fn start(mut self) -> Result<Vec<u8>, ManagerWrapperError<ManagerT, Self>> {
        self.send_request()?;
        let res = || -> Result<Vec<u8>, ManagerWrapperError<ManagerT, Self>> {
            while let Some(resp) = self.server.get_response()? {
                self.handle_response(resp.0, resp.1)?;
            }
            unreachable!()
            //Ok(self.manager.move_data())
        }();

        if let Err(ManagerWrapperError::ManagerError(ManagerError::TheDataIsFilled)) = res {
            println!("Finished");
            Ok(self.manager.move_data())
        } else {
            res
        }
    }
}

use sha2::Sha256;

use my_hex::ToHex;

fn hash_to_string(data: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&data);

    let res = hasher.finalize();

    char::encode(res.into_iter())
}

pub fn test_with_server<M: Manager>(server_addr: &str) -> String {
    let (sc, (r, s)) = ServerCommunicator::new(server_addr).unwrap();
    sc.start();
    let client = Client::new(s, r).unwrap();
    let data_len = client.get_data_len();
    let bm = RealManagerWrapper {
        server: client,
        manager: M::init(data_len),
    };

    //hash
    let res = bm.start().unwrap();

    hash_to_string(res)
}
