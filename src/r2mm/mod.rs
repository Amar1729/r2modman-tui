/// Module for file management of the ror2 mods.

pub mod launcher;

use std::fs;
use std::io;
use std::path;

use zip::{ZipArchive, result::ZipResult};

use hyper;

use serde::Deserialize;
use serde_json;

use unicode_bom::Bom;

use crate::response::Package;

// hard coded for now
// eventually, will use a Config and xdg config
const DIR: &'static str = "/tmp/mods";

/// manifest.json for a plugin
#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    pub name: String,
    /// major.minor.rev:
    version_number: String,
    website_url: String,
    description: String,
    dependencies: Vec<String>,
}

pub fn check_pkg(pkg: Package) -> bool {
    let dl_dir = path::Path::new(DIR);
    return dl_dir.join(pkg.latest.full_name).exists();
}

/// Gives a Package matching the given Manifest's name.
pub fn pkg_from_manifest(m: Manifest, pkgs: Vec<Package>) -> Option<Package> {
    if let Some(pkg) = pkgs.iter().find(|p| {p.name == m.name}) {
        Some(pkg.clone())
    } else {
        None
    }
}

// pub fn manifest_from_pkg(p: Package) -> Option<Manifest> {
// }

pub fn get_local_pkgs() -> io::Result<Vec<Manifest>> {
    let dl_dir = path::Path::new(DIR);
    match fs::read_dir(dl_dir) {
        Ok(files) => {
            let mut pkgs: Vec<Manifest> = vec![];
            for plugin in files {
                let p = plugin?.path();
                if path::Path::new(&p).is_dir() {
                    let file = path::Path::new(&p).join("manifest.json");

                    if let Ok(bom) = file.to_str().unwrap().parse::<Bom>() {
                        let mut content = fs::read_to_string(file)?;
                        content.drain(..bom.len());

                        match serde_json::from_str::<Manifest>(&content) {
                            Ok(m) => {
                                pkgs.push(m);
                            }
                            Err(e) => { eprintln!("err: {}", e); }
                        }
                    }
                }
            }

            Ok(pkgs)
        }
        Err(_) => { Ok(vec![]) }
    }
}

/// Unzips a zip package contained in 'content' to a directory specified by 'name'
#[allow(deprecated)]
pub fn unzip_pkg(name: String, content: hyper::body::Bytes) -> ZipResult<()> {
    let directory = path::Path::new(DIR).join(name);

    let reader = io::Cursor::new(content);
    let mut zip = ZipArchive::new(reader).unwrap();

    /*
    // https://github.com/zip-rs/zip/pull/116
    zip
        .extract(&path::PathBuf::from(format!("/tmp/mods/{}", name)))
        .expect("failed");
    */

    if directory.exists() {
        // for now, assume previous downloads have completed successfully
        return Ok(());
    }

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let filepath = file.sanitized_name();

        let outpath = directory.join(filepath);

        if (file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
