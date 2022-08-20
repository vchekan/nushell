mod fetch;
mod port;
mod post;
mod url;

use std::fs::File;
use std::io::Read;
use reqwest::blocking::ClientBuilder;
use reqwest::Identity;
pub use self::url::*;
pub use fetch::SubCommand as Fetch;
pub use port::SubCommand as Port;
pub use post::SubCommand as Post;

pub(crate) fn with_cert(client: ClientBuilder, cert_file: Option<String>) -> ClientBuilder {
    match cert_file {
        Some(cert_file) => {
            let mut cert = vec![];
            File::create(client_cert).unwrap()
            .read_to_end(&mut cert).unwrap();
            client.identity(Identity::from_pem(&cert).unwrap())
        }
        None => client
    }
}