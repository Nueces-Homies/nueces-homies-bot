use std::fs;
use std::path::Path;

pub fn unzip(src_zip: &Path, dst_dir: &Path) -> color_eyre::Result<()> {
    let zip_file = fs::File::open(src_zip)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)?;
        let filename = match file_in_zip.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let dst_path = dst_dir.join(filename);
        let mut dst_file = fs::File::create(&dst_path)?;
        std::io::copy(&mut file_in_zip, &mut dst_file)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dst_path, fs::Permissions::from_mode(0o775))?;
        }
    }

    Ok(())
}
