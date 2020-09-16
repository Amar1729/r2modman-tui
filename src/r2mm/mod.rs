/// Module for file management of the ror2 mods.

use std::fs;
use std::io;
use std::path;

use zip::{ZipArchive, result::ZipResult};

use hyper;

pub fn count_pkgs() -> usize {
    let dl_dir = path::Path::new("/tmp/mods");

    fs::read_dir(dl_dir).unwrap().count()
}

/// Unzips a zip package contained in 'content' to a directory specified by 'name'
pub fn unzip_pkg(name: String, content: hyper::body::Bytes) -> ZipResult<()> {
    let directory = path::Path::new("/tmp/mods").join(name);

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
