#![allow(unreachable_code)]

use std::fs;
use crate::response::{Resp, Package};

extern crate hyper;
extern crate hyper_tls;

use hyper::body;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;

pub async fn get_pkgs(url: Uri) -> Vec<Package> {
    // read from fs while testing
    return get_pkgs_dbg(url);

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    match client.get(url).await {
        Ok(res) => {
            let body_bytes = body::to_bytes(res.into_body()).await.unwrap();
            let body = String::from_utf8(body_bytes.to_vec()).expect("not valid utf8");

            let resp: Resp = serde_json::from_str(&body).unwrap();
            resp.results
        },
        Err(err) => panic!("unhandled: {}", err),
    }
}

pub fn get_pkgs_dbg(url: Uri) -> Vec<Package> {
    let body = fs::read_to_string("resp.json").unwrap();
    let resp: Resp = serde_json::from_str(&body).unwrap();
    resp.results
}
