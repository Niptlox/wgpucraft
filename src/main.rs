#[cfg(feature = "tracy")]
use tracy_client::Client;
use wgpucraft::launcher::run;

fn main() {
    #[cfg(feature = "tracy")]
    let _client = Client::start(); // Inicia el cliente de Tracy

    run();
}
