use std::io::{copy, Read, Write};
use std::env;
use std::fmt;
use std::result::Result as StdResult;

use errors::*;

use hyper;
use url::ParseError as UrllibParseError;
use hyper::Url;
use hyper::status::StatusCode;
use hyper::client::{Client, IntoUrl, Response};
use hyper::header::{Header, HeaderFormat};
use hyper::error::Result as HResult;

use serde::de::Deserialize;

/// Represents vault authorization token string
#[derive(Clone, Debug)]
pub struct VaultToken(String);

impl VaultToken {
    pub fn from_env() -> Result<VaultToken> {
        env::var("VAULT_TOKEN").chain_err(|| "Environment variable VAULT_TOKEN not found").map(VaultToken)
    }
}

impl Header for VaultToken {
    fn header_name() -> &'static str {
        "X-Vault-Token"
    }

    fn parse_header(raw: &[Vec<u8>]) -> HResult<Self> {
        // TODO: investigate if vault returns this header somewhere
        // Implement parsing if it does
        Err(hyper::Error::Header)
    }
}

impl From<String> for VaultToken {
    fn from(s: String) -> Self {
        VaultToken(s)
    }
}

impl HeaderFormat for VaultToken {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Vault base path
#[derive(Debug, Clone)]
pub struct VaultAddress(Url);

impl VaultAddress {
    pub fn new<U: IntoUrl>(addr: U) -> Result<Self> {
        let url = try!(addr.into_url().chain_err(|| ErrorKind::UrlParseError));
        let mut addr = VaultAddress(url);
        try!(addr.set_version("v1"));
        Ok(addr)
    }

    pub fn from_env() -> Result<Self> {
        let addr = try!(env::var("VAULT_ADDR"));
        addr.into_url()
            .chain_err(|| ErrorKind::UrlParseError)
            .map(VaultAddress)
    }

    // Sets the API version (YAGNI?)
    fn set_version(&mut self, version: &str) -> Result<()> {
        self.0 = try!(self.0.join(version).chain_err(|| ErrorKind::UrlParseError));
        Ok(())
    }

    fn join(&self, path: &str) -> Result<Url> {
        let mut url = self.0.clone();
        {
            let mut segs = try!(url.path_segments_mut().map_err(|_| ErrorKind::UrlParseError));
            segs.push(path);
        }
        Ok(url)
    }
}

impl IntoUrl for VaultAddress {
    fn into_url(self) -> StdResult<Url, UrllibParseError> {
        Ok(self.0)
    }
}

/// API for authorized client
#[derive(Clone, Debug)]
pub struct VaultClient {
    addr: VaultAddress,
    token: VaultToken,
}

impl VaultClient {
    /// Creates new vault client
    pub fn new<U: IntoUrl>(addr: U, token: String) -> Result<Self> {
        let addr = try!(VaultAddress::new(addr));
        Ok(VaultClient {
            addr: addr,
            token: token.into(),
        })
    }

    /// Creates new vault client using environment variables for configuration
    /// Currently supported variables: `VAULT_ADDR`, `VAULT_TOKEN`
    pub fn from_env() -> Result<Self> {
        let addr = try!(VaultAddress::from_env());
        let token = try!(VaultToken::from_env());
        VaultClient::new(addr, token.0)
    }

    /// Lower level secret getter. Returns HTTP response without checking return codes etc
    pub fn get_secret_raw(&self, path: &str) -> Result<Vec<u8>> {
        let client = Client::new();
        let url = try!(self.addr.join(path));
        // TODO: logging
        println!("{:?}", url);
        let mut response = try!(client.get(url).header(self.token.clone()).send().chain_err(|| "Request error"));
        let mut secret = Vec::new();
        let _: usize = try!(response.read_to_end(&mut secret).chain_err(|| "Request reading error"));
        match response.status {
            StatusCode::Ok => Ok(secret),
            StatusCode::NoContent => Ok(secret),
            _ => Err(ErrorKind::HttpError(response.status_raw().clone()).into()),
        }
    }
}
