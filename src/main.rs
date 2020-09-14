mod client;
mod response;
mod interface;
mod util;

use hyper::Uri;

#[tokio::main]
async fn main() {
    let url: Uri = "https://thunderstore.io/api/v2/package/"
        .parse()
        .unwrap();

    let pkgs = client::get_pkgs(url).await;
    interface::start_app(pkgs).await;
}
