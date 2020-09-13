extern crate serde_json;

extern crate hyper;
extern crate hyper_tls;

use hyper::body;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Latest {
    name: String,
    full_name: String,
    description: String,
    icon: String,
    version_number: String, // major.minor.rev
    dependencies: Vec<String>,
    download_url: String,
    downloads: u64,
    date_created: String,
    website_url: String,
    is_active: bool,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    full_name: String,
    owner: String,
    package_url: String,
    date_created: String,
    date_updated: String,
    rating_score: u8,
    is_pinned: bool,
    is_deprecated: bool,
    total_downloads: u64,
    latest: Latest,
}

#[derive(Deserialize, Debug)]
struct Resp {
    count: u16,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<Package>,
}

async fn get(url: Uri) -> hyper::Result<String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    match client.get(url).await {
        Ok(res) => {
            let body_bytes = body::to_bytes(res.into_body()).await?;
            let body = String::from_utf8(body_bytes.to_vec()).expect("not valid utf8");

            Ok(body)
        },
        Err(err) => panic!("unhandled: {}", err),
    }
}

#[tokio::main]
async fn main() {
    let url: Uri = "https://thunderstore.io/api/v2/package/"
        .parse()
        .unwrap();

    match get(url).await {
        Ok(json) => {
            let resp: Resp = serde_json::from_str(&json).unwrap();

            for pkg in resp.results {
                println!("{}", pkg.full_name);
            }
        },
        Err(e) => panic!("errored with: {}", e),
    }
}
