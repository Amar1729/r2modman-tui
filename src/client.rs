#![allow(unreachable_code)]

use std::fs;
use std::path;
use std::io;

use crate::response::{Resp, Package};
use crate::r2mm;

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

pub fn check_pkg(pkg: Package) -> bool {
    let dl_dir = path::Path::new("/tmp/mods");
    let zipfile = format!("{}.zip", pkg.latest.full_name);

    return dl_dir.join(zipfile).exists();
}

/// Downloads a package to /tmp/mods
/// TODO - download deps as well
/// TODO - specify the path in a config file
/// TODO - turn this into a general Downloader with methods
/// TODO - a Downloader would use one Client to take advantage of advantage caching
pub async fn download_pkg(pkg: Package) -> Result<(), &'static str> {
    let download_url = pkg.latest.download_url;

    /*
    let deps = pkg.latest.dependencies
        .iter()
        .map(|dep_name| {})
        .collect();
    */

    // let client = reqwest::Client::new();
    let response = reqwest::get(&download_url).await.unwrap();

    match response.bytes().await {
        Ok(bytes) => {
            match r2mm::unzip_pkg(pkg.latest.full_name, bytes) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        Err(_) => {}
    }

    Ok(())
}

// async fn download(url: Uri, pth: String) -> Result<(), &'static str> {
// }
