#[cfg(feature = "core")]
pub mod asset_server;
mod save_server;

#[cfg(feature = "core")]
pub fn dynamic_hasher(val: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in val.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(feature = "package")]
pub mod asset_package {
    use fs_extra::dir::CopyOptions;
    use std::path::PathBuf;
    use std::{env, fs};

    pub fn append_assets(package_path: &str, package_name: &str) -> std::io::Result<()> {
        let mut base_asset_path =
            env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set");
        base_asset_path.push_str("/assets/packages/");
        base_asset_path.push_str(package_name);
        let base_asset_path = PathBuf::from(base_asset_path);
        let mut base_dev_asset_path =
            env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set");
        base_dev_asset_path.push_str("/dev-assets/packages/");
        base_dev_asset_path.push_str(package_name);
        let base_dev_asset_path = PathBuf::from(base_dev_asset_path);

        let mut package_asset_path = String::from(package_path);
        package_asset_path.push_str("/assets");
        let package_asset_path = PathBuf::from(package_asset_path);
        let mut package_dev_asset_path = String::from(package_path);
        package_dev_asset_path.push_str("/dev-assets");
        let package_dev_asset_path = PathBuf::from(package_dev_asset_path);

        fs::create_dir_all(&base_asset_path).unwrap();
        fs::create_dir_all(&base_dev_asset_path).unwrap();

        if package_asset_path.exists() {
            if base_asset_path.exists() {
                fs::remove_dir_all(&base_asset_path)?;
            }

            let mut options = CopyOptions::new();
            options.copy_inside = true;

            fs_extra::dir::copy(package_asset_path, &base_asset_path, &options)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        if package_dev_asset_path.exists() {
            if base_dev_asset_path.exists() {
                fs::remove_dir_all(&base_dev_asset_path)?;
            }

            let mut options = CopyOptions::new();
            options.copy_inside = true;

            fs_extra::dir::copy(package_dev_asset_path, &base_dev_asset_path, &options)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Ok(())
    }
}
