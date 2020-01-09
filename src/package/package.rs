use std::fs::{copy, create_dir_all, remove_file, remove_dir, File};
use std::path::PathBuf;

use regex::Regex;
use log::{error, info};
use serde_derive::Deserialize;
use lazy_static::lazy_static;

const MANIFEST: &'static str = "manifest.toml";


lazy_static! {
    static ref MANIFEST_NAME: Regex = Regex::new(r"^[a-zA-Z0-9]+[a-zA-Z0-9\-_]*$").unwrap();
}

use crate::errors::Result;

pub enum RootPath {
    Local(PathBuf),
    Remote(String),
}

#[derive(Deserialize, Debug)]
pub struct Package {
    name: String,
    description: String,
    usage: String,
    version: String,

    pub files: Vec<String>,
}

impl Package {
    pub fn safe_name(&self) -> String {
        self.name.to_lowercase()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn load(&self, src: RootPath, dst: PathBuf) {
        let mut copied_files = Vec::new();

        for file in &self.files {
            let dst_full = dst.join(&file);
            if let Some(path) = create_dir(&dst_full) {
                copied_files.push(path);
            }

            let res = match src {
                RootPath::Local(ref path) => copy_local(&path.join(&file), &dst_full),
                RootPath::Remote(ref url) => copy_remote(url, &file, &dst_full),
            };

            match res {
                Ok(_) => copied_files.push(dst_full),
                Err(e) => {
                    error!("Failed to copy file: {:?}, Rolling back", e);
                    rollback(copied_files);
                    return;
                }
            }
        }
    }

    pub fn update(&self, src: RootPath, dst: PathBuf) {
        // Only update existing packages
        if !dst.exists() {
            return
        }

        // Check the local manifest for a lower version
        if !self.version_is_increased(&dst) {
            return
        }

        let dst_full = dst.join(MANIFEST);
        let _res = match src {
            RootPath::Local(ref path) => copy_local(&path.join(MANIFEST), &dst_full),
            RootPath::Remote(ref url) => copy_remote(url, MANIFEST, &dst_full),
        };

        self.load(src, dst);
        println!("Updated: {}", self.name);
    }

    pub fn install(&self, src: RootPath, dst: PathBuf) {
        // Copy the manifest
        let dst_full = dst.join(MANIFEST);
        let _res = match src {
            RootPath::Local(ref path) => copy_local(&path.join(MANIFEST), &dst_full),
            RootPath::Remote(ref url) => copy_remote(url, MANIFEST, &dst_full),
        };

        // Only install new packages
        let existing = self.existing_files(&dst);

        if existing.len() > 0 {
            for file in existing {
                info!("file exists: {}", file);
            }
            println!("already installed {}", self.name);
            return;
        }

        self.load(src, dst);
        println!("Installed: {}", self.name);
    }

    pub fn is_valid(&self) -> bool {
        if !MANIFEST_NAME.is_match(&self.name) {
            eprintln!("Invalid name \"{}\" (only alphanmeric characters, '-' and '_')", self.name);
            return false;
        }

        match self.version.parse::<f32>() {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Invalid version \"{}\" (version should be #.#, e.g 1.3)", self.version);
                return false
            }
        }
        true
    }

    fn existing_files(&self, dst: &PathBuf) -> Vec<String> {
        let mut files = Vec::new();
        for file in &self.files {
            if dst.join(&file).exists() {
                files.push(file.clone());
            }
        }
        files
    }

    /// Compare the remote version with the local version
    fn version_is_increased(&self, dst: &PathBuf) -> bool {
        match super::local_package(dst.join(MANIFEST)) {
            Ok(local) => {
                match (
                    local.version.parse::<f32>(),
                    self.version.parse::<f32>()
                ) {
                    (Ok(local), Ok(remote)) => {
                        eprintln!("remote: {:?}", remote);
                        eprintln!("local : {:?}", local);
                        return remote > local
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("Error loading local manifest: {:?}", e);
            }
        }

        return false
    }
}

fn create_dir(dir: &PathBuf) -> Option<PathBuf> {
    if let Some(parent_dir) = dir.parent() {
        if create_dir_all(parent_dir).is_ok() {
            return Some(parent_dir.into());
        }
    }
    return None;
}

fn copy_local(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    copy(src, dst)?;
    Ok(())
}

fn copy_remote(root_url: &str, fragment: &str, dst: &PathBuf) -> Result<()> {
    let complete_url = format!("{}{}", root_url, fragment);

    let mut response = reqwest::blocking::get(&complete_url)?;

    match response.status().as_u16() {
        200..=299 => {}
        status => {
            if status == 404 {
                return Err(crate::errors::Error::FileNotFound(fragment.to_string()));
            } else {
                return Err(crate::errors::Error::InvalidResponse);
            }
        }
    }

    let mut file = File::create(dst)?;
    let _ = response.copy_to(&mut file);

    Ok(())
}

fn rollback(paths: Vec<PathBuf>) {
    paths.iter().filter(|file| file.is_file()).for_each(|file| {
        let _ = remove_file(file);
    });

    paths.iter().filter(|dir| dir.is_dir()).for_each(|dir| {
        let _ = remove_dir(dir);
    });
}
