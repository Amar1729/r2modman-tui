use std::fs;
use std::path;

pub fn count_pkgs() -> usize {
    let dl_dir = path::Path::new("/tmp/mods");

    fs::read_dir(dl_dir).unwrap().count()
}
