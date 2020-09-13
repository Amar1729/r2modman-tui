mod client;
mod response;

use client::get_pkgs;

use hyper::Uri;

#[tokio::main]
async fn main() {
    let url: Uri = "https://thunderstore.io/api/v2/package/"
        .parse()
        .unwrap();

    for pkg in get_pkgs(url).await {
        println!("{}", pkg.full_name);
    }
}
