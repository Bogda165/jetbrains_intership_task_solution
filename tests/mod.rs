mod test_utils;

#[cfg(test)]
mod tests {
    use crate::{test_utils::server::*, test_utils::test_wrapper::*};
    use interval_list::core::Chunk;

    use managers::{basic_manager::BasicManager, random_manager::RandomManager};
    #[test]
    pub fn test_server_basic() {
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

    #[test]
    pub fn test_server_random() {
        let server = Server::init_with_lower_bound(50);
        for _ in 0..100 {
            let server = server.clone();
            let mut tm = TestManagerWrapper {
                mangaer: RandomManager::new(server.get_len() as usize, 10),
                server,
            };

            tm.send_request();

            let dl = tm.server.get_len();

            println!("Data len: {}", dl);
            println!("Server data: {:?}", tm.server.data);
            println!("Recieved data: {:?}", tm.mangaer.data);

            assert!(
                tm.mangaer
                    .filled_list
                    .get_complement_intervals(
                        Chunk::new(0, tm.server.get_data_len()).expect("Chunk createiong")
                    )
                    .expect("complement intervals creation")
                    .len()
                    == 0
            );

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
}
