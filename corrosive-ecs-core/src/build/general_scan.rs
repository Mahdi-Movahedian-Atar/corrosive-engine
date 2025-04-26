use crate::build::ENGINE_DIR;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
pub enum ModifiedState {
    #[default]
    Changed,
    Removed,
    None,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PathMap {
    pub path: PathBuf,
    pub modified_time: SystemTime,
    pub modified_state: ModifiedState,
    pub sub_maps: Vec<PathMap>,
}

impl Default for PathMap {
    fn default() -> Self {
        PathMap {
            path: Path::new("./").to_path_buf(),
            modified_time: SystemTime::now(),
            sub_maps: Vec::new(),
            modified_state: ModifiedState::Changed,
        }
    }
}

impl PathMap {
    fn remove(&mut self) {
        self.modified_state = ModifiedState::Removed;
        for m in &mut self.sub_maps {
            m.modified_state = ModifiedState::Removed;
            m.remove()
        }
    }

    fn none(&mut self) {
        self.modified_state = ModifiedState::None;
        for m in &mut self.sub_maps {
            m.modified_state = ModifiedState::None;
            m.none()
        }
    }
}

pub fn get_path_map(file_path: &str, default_path: &str) -> PathMap {
    match fs::read_to_string(file_path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => PathMap {
            path: Path::new(default_path).to_path_buf(),
            modified_time: SystemTime::now(),
            sub_maps: Vec::new(),
            modified_state: ModifiedState::Changed,
        },
    }
}

pub fn write_path_map(path_map: &PathMap, path: &str) -> io::Result<()> {
    let serialized = serde_json::to_string_pretty(path_map)?;
    fs::write(path, serialized)?;
    Ok(())
}

pub fn scan_directory(path_map: &mut PathMap, start_path: &str) -> io::Result<()> {
    if path_map.path.as_path().is_dir() && !path_map.path.as_path().ends_with(ENGINE_DIR) {
        let mut files = Vec::new();
        for entry in fs::read_dir(path_map.path.as_path())? {
            let entry = entry?;
            let path = entry.path();
            let meta_data = fs::metadata(&path)?;
            files.push(path.clone());

            match path_map.sub_maps.iter_mut().find(|item| item.path == path) {
                Some(T) => {
                    if T.modified_time != meta_data.modified()? {
                        T.modified_time = meta_data.modified()?;
                        T.modified_state = ModifiedState::Changed;
                    } else {
                        T.none()
                    }
                    scan_directory(T, start_path)?
                }
                None => {
                    path_map.sub_maps.push(PathMap {
                        path: path.clone(),
                        modified_time: meta_data.modified()?,
                        modified_state: ModifiedState::Changed,
                        sub_maps: vec![],
                    });
                    scan_directory(path_map.sub_maps.last_mut().unwrap(), start_path)
                }?,
            }
        }
        let _ = path_map
            .sub_maps
            .iter_mut()
            .filter(|item| !files.contains(&item.path))
            .for_each(|item| item.remove());
        path_map
            .sub_maps
            .retain(|item| item.modified_state != ModifiedState::Removed);
    }
    if path_map.path.as_path() == Path::new(start_path) {
        path_map.modified_state = ModifiedState::None;
        path_map
            .sub_maps
            .iter()
            .find(|item| item.modified_state == ModifiedState::Changed)
            .into_iter()
            .for_each(|_| path_map.modified_state = ModifiedState::Changed);
    }
    Ok(())
}