//! Commandline Mod Manager for Risk of Rain 2 video game.

mod client;
mod response;
mod interface;
mod util;
mod r2mm;

extern crate serde;
extern crate serde_json;

extern crate hyper;
extern crate hyper_tls;

use hyper::Uri;

#[tokio::main]
async fn main() {
    let url: Uri = "https://thunderstore.io/api/v2/package/"
        .parse()
        .unwrap();

    let pkgs = client::get_pkgs(url).await;
    // interface::start_app(pkgs).await;
    r2mm::launcher::launch_game(false);
    // let pkg = pkgs[1].clone();
    // client::download_pkg(pkg, pkgs).await;

    // println!("result: {}", r2mm::get_local_pkgs().unwrap().len());
}
