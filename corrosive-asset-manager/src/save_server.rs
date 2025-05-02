/*use std::any::Any;
use std::cell::LazyCell;
use std::fs;
use std::path::{Path, PathBuf};
use bincode;
use aes_gcm;

pub enum SaveLocationType{
    Setting,
    ProfileLevel,
    SlotLevel
}
pub enum SaveSafetyType {
    JSON,
    Bits,
    Encrypted
}
pub enum SaveError {
    SerializationError(bincode::Error),
    EncryptionError(aes_gcm::Error),
    StorageError(String),
    MigrationFailed,
}

pub trait SaveObject: Sized {
    fn default() -> Option<Self> {
        Some(Default::default())
    }
    fn ident() -> String;
    fn migrate() -> Option<Self> {
        None
    }
    fn save_location_type() -> SaveLocationType {
        SaveLocationType::SlotLevel
    }
    fn save_safety_type() -> SaveSafetyType {
        SaveSafetyType::JSON
    }
}

static mut CURRENT_PROFILE: Option<String> = None;
static mut CURRENT_SLOT: Option<String> = None;

static ABSOLUTE_PATH: LazyCell<PathBuf> = LazyCell::new(||{
    #[cfg(windows)]{
        let mut path = dirs::data_dir().expect("Failed to get user data directory");
        path.push(Path::new(env!("CARGO_PKG_NAME")));
        path
    }
});

pub struct SaveServer{

}

fn ensure_dir_exists(path_buf: &PathBuf) -> Result<(), SaveError> {
    if !path_buf.exists() {
        fs::create_dir_all(&path_buf)
            .map_err(|e| SaveError::StorageError(e.to_string()))?;
    }
    Ok(())
}
impl<T:SaveObject> SaveServer{
    pub fn load_absolute(profile: &str, slot: &str) -> Result<T, SaveError>{
        let mut path = ABSOLUTE_PATH.clone();
        path.push(profile);
        path.push(slot);
        ensure_dir_exists(&path)?;
        let mut ident = T::ident();
        match T::save_safety_type() {
            SaveSafetyType::JSON => {
                ident.push_str("json");
                path.push(ident);
                let data =match fs::read(path).ok() {
                    None => {
                        Err(SaveError::StorageError("Failed to read file".to_string()))
                    }
                    Some(t) => {Ok(t)}
                }?;
                Ser
            }
            SaveSafetyType::Bits => {}
            SaveSafetyType::Encrypted => {}
        }
        if T::ident() {  }
        let data = std::fs::read(path).ok()?;
    }
}*/
