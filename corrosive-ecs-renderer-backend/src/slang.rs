use reqwest::blocking::get;
use std::collections::HashSet;
use std::error::Error;
use std::fs::Metadata;
use std::{
    env,
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
    process::Command,
};
use tar::Archive;
use walkdir::WalkDir;
use zip::ZipArchive;

const SLANG_VERSION: &str = "2025.6.3";
const SLANG_REPO: &str = "https://github.com/shader-slang/slang/releases/download";

#[derive(Debug)]
enum SlangError {
    Io(io::Error),
    Reqwest(reqwest::Error),
    CompilationFailed,
}

impl From<io::Error> for SlangError {
    fn from(err: io::Error) -> Self {
        SlangError::Io(err)
    }
}

impl From<reqwest::Error> for SlangError {
    fn from(err: reqwest::Error) -> Self {
        SlangError::Reqwest(err)
    }
}

fn ensure_slang_installed() -> Result<PathBuf, SlangError> {
    let target_dir =
        PathBuf::from(env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set"))
            .join("dev-assets/packages/slang");
    fs::create_dir_all(&target_dir)?;
    let bin_name = if cfg!(windows) {
        "slangc.exe"
    } else {
        "slangc"
    };
    let bin_path = target_dir.join("bin").join(bin_name);

    if !bin_path.exists() {
        println!("Downloading Slang compiler...");

        let (os, ext) = if cfg!(windows) {
            ("windows", "zip")
        } else if cfg!(target_os = "macos") {
            ("macos", "tar.gz")
        } else {
            ("linux", "tar.gz")
        };

        let url = format!(
            "{}/v{}/slang-{}-{}-x86_64.{}",
            SLANG_REPO, SLANG_VERSION, SLANG_VERSION, os, ext
        );
        let response = get(&url)?;
        let bytes = response.bytes()?;

        if cfg!(windows) {
            let mut zip = ZipArchive::new(io::Cursor::new(bytes)).expect("Could not load zip");
            zip.extract(&target_dir).expect("Could not extract zip");
        } else {
            let mut archive = Archive::new(io::Cursor::new(bytes));
            archive.unpack(&target_dir)?;
        }
    }

    Ok(bin_path)
}
pub(crate) struct ShaderManager {
    dev_assets: PathBuf,
    assets: PathBuf,
}

impl ShaderManager {
    pub fn new() -> Self {
        let assets =
            PathBuf::from(env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set"))
                .join("assets/");
        let dev_assets =
            PathBuf::from(env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set"))
                .join("dev-assets/");
        fs::create_dir_all(&assets).unwrap();
        fs::create_dir_all(&dev_assets).unwrap();
        Self { dev_assets, assets }
    }

    pub fn sync_shaders(&self) -> Result<(), Box<dyn Error>> {
        let (slang_files, wgsl_files) = self.scan_directories()?;
        self.process_deletions(&wgsl_files, &slang_files)?;
        self.process_compilations(&slang_files, &wgsl_files)
    }

    fn scan_directories(&self) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), Box<dyn Error>> {
        let mut slang_files = HashSet::new();
        let mut wgsl_files = HashSet::new();

        for entry in WalkDir::new(&self.dev_assets) {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry.path().extension().map_or(false, |e| e == "slang")
                && !entry
                    .path()
                    .file_stem()
                    .map_or(false, |s| s.to_string_lossy().ends_with(".lib"))
            {
                slang_files.insert(self.relative_path(entry.path(), &self.dev_assets)?);
            }
        }

        for entry in WalkDir::new(&self.assets) {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry.path().extension().map_or(false, |e| e == "wgsl")
                && entry
                    .path()
                    .file_stem()
                    .map_or(false, |s| s.to_string_lossy().ends_with(".slang"))
            {
                wgsl_files.insert(self.relative_path(entry.path(), &self.assets)?);
            }
        }

        Ok((slang_files, wgsl_files))
    }

    fn process_deletions(
        &self,
        wgsl_files: &HashSet<PathBuf>,
        slang_files: &HashSet<PathBuf>,
    ) -> Result<(), Box<dyn Error>> {
        for spirv_path in wgsl_files {
            let expected_slang = spirv_path.with_file_name(spirv_path.file_stem().unwrap());

            if !slang_files.contains(&expected_slang) {
                let full_path = self.assets.join(spirv_path);
                println!("Removing orphaned WGSL: {}", full_path.display());
                fs::remove_file(full_path)?;
            }
        }
        Ok(())
    }

    fn process_compilations(
        &self,
        slang_files: &HashSet<PathBuf>,
        wgsl_files: &HashSet<PathBuf>,
    ) -> Result<(), Box<dyn Error>> {
        for slang_path in slang_files {
            let spirv_path = {
                let mut p = slang_path.to_path_buf();
                p.set_extension("slang.wgsl");
                p
            };

            let needs_compile = match wgsl_files.contains(&spirv_path) {
                true => self.needs_recompile(slang_path, &spirv_path)?,
                false => true,
            };

            if needs_compile {
                self.compile_shader(slang_path, &spirv_path)?;
            }
        }
        Ok(())
    }

    fn needs_recompile(&self, slang_path: &Path, wgsl_path: &Path) -> Result<bool, Box<dyn Error>> {
        let dev_meta = self.metadata(&self.dev_assets.join(slang_path))?;
        let asset_meta = self.metadata(&self.assets.join(wgsl_path))?;

        Ok(dev_meta.modified()? > asset_meta.modified()?)
    }

    fn compile_shader(&self, slang_path: &Path, wgsl_path: &Path) -> Result<(), Box<dyn Error>> {
        let full_slang = self.dev_assets.join(slang_path);
        let full_wgsl = self.assets.join(wgsl_path);

        if let Some(parent) = full_wgsl.parent() {
            fs::create_dir_all(parent)?;
        }

        println!("Compiling {}...", full_slang.display());

        Command::new(ensure_slang_installed().expect("Failed to ensure Slang is installed"))
            .arg("-o")
            .arg(full_wgsl)
            .arg("-target")
            .arg("wgsl")
            .arg(full_slang)
            .status()?;

        Ok(())
    }

    fn relative_path(&self, path: &Path, base: &Path) -> Result<PathBuf, Box<dyn Error>> {
        Ok(path.strip_prefix(base)?.to_path_buf())
    }

    fn metadata(&self, path: &Path) -> Result<Metadata, Box<dyn Error>> {
        Ok(fs::metadata(path)?)
    }
}
