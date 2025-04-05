mod app;
mod client;
pub mod managers;
mod real_manager_wrapper;

use app::Application;

fn main() -> Result<(), std::io::Error> {
    let app = Application::new()?;

    println!("{:?}", app);

    app.start()
}
