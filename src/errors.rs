use std;

use hyper;
use serde_json;
use url;

error_chain!{

    links {
    }

    foreign_links {
        std::io::Error, IoError;
        hyper::error::Error, HyperError;
        serde_json::error::Error, DeserializationError;
        std::env::VarError, EnvError;
    }

    errors {
            //UrlParseError(t: url::ParseError)
            UrlParseError
            HttpError(t: hyper::http::RawStatus)
    }
}
