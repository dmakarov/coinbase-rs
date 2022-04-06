extern crate base64;
extern crate failure;
extern crate futures;
extern crate hmac;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate tokio;
extern crate tokio_stream;
extern crate uritemplate;

pub mod error;
pub mod private;
pub mod public;
pub mod request;

pub use error::CBError;
pub use private::Private;
pub use public::Public;

pub const MAIN_URL: &str = "https://api.coinbase.com";

pub use uuid::Uuid;

pub type DateTime = chrono::DateTime<chrono::Utc>;

pub type Result<U> = std::result::Result<U, CBError>;
