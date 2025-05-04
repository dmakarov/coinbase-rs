use std::collections::HashMap;
use std::result;

use http::{request, Method, Request, Uri, Version};
use hyper::Body;
use jwt_simple::prelude::*;

#[derive(Debug)]
pub struct Error {}

pub type Result<T> = result::Result<T, Error>;

const USER_AGENT: &str = concat!("coinbase-rs/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, Default)]
pub struct Parts {
    /// The request's method
    pub method: Method,

    /// The request's URI
    pub uri: Uri,

    /// The request's version
    pub version: Version,

    /// The request's headers
    pub headers: HashMap<String, String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Payload {
    uri: String,
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    auth: Option<(String, String)>,
    parts: Parts,
    body: Vec<u8>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            auth: None,
            parts: Parts {
                method: Method::GET,
                uri: "/".parse().unwrap(),
                version: Version::default(),
                headers: HashMap::new(),
            },
            body: Vec::new(),
        }
    }

    pub fn new_with_auth(key: &str, secret: &str) -> Builder {
        Builder {
            auth: Some((key.to_string(), secret.to_string())),
            parts: Parts {
                method: Method::GET,
                uri: "/".parse().unwrap(),
                version: Version::default(),
                headers: HashMap::new(),
            },
            body: Vec::new(),
        }
    }

    pub fn method(self, method: Method) -> Builder {
        let mut _self = self;
        _self.parts.method = method;
        _self
    }

    pub fn uri(self, uri: Uri) -> Builder {
        let mut _self = self;
        _self.parts.uri = uri;
        _self
    }

    pub fn version(self, version: Version) -> Builder {
        let mut _self = self;
        _self.parts.version = version;
        _self
    }

    pub fn header(self, key: &str, value: &str) -> Builder {
        let mut _self = self;
        _self.parts.headers.insert(key.into(), value.into());
        _self
    }

    pub fn body(self, body: &[u8]) -> Builder {
        let mut _self = self;
        _self.body = body.to_owned();
        _self
    }

    pub fn build(self) -> Request<Body> {
        let _self = if let Some((ref key, ref secret)) = self.auth {
            let path = format!(
                "{}{}",
                self.parts.uri.host().unwrap(),
                self.parts.uri.path_and_query().unwrap(),
            );
            let token = Self::token(key, secret, &self.parts.method, &path);
            let bearer = format!("Bearer {token}");
            self.clone()
                .header("User-Agent", USER_AGENT)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Authorization", &bearer)
        } else {
            self
        };
        let mut builder = request::Builder::new()
            .method(_self.parts.method)
            .uri(_self.parts.uri);
        for (key, value) in _self.parts.headers {
            builder = builder.header(&key, &value);
        }
        builder.body(_self.body.into()).unwrap()
    }

    fn token(key_name: &str, secret: &str, method: &Method, path: &str) -> String {
        let pkey = match elliptic_curve::SecretKey::<p256::NistP256>::from_sec1_pem(secret) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to load private key from pem: {e}");
                return String::default();
            }
        };
        let key_pair = match jwt_simple::prelude::ES256KeyPair::from_bytes(&pkey.to_bytes()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to load key_pair from bytes: {e}");
                return String::default();
            }
        };
        let key_pair = key_pair.with_key_id(key_name);
        let payload = Payload {
            uri: format!("{} {}", method.as_str(), path),
        };
        let mut claims = jwt_simple::claims::Claims::with_custom_claims(
            payload,
            coarsetime::Duration::from_secs(120),
        )
        .with_issuer("cdp".to_string())
        .with_subject(key_name);
        claims.create_nonce();
        let token = match key_pair.sign(claims) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to sign claims: {e}");
                return String::default();
            }
        };
        token.to_string()
    }
}
