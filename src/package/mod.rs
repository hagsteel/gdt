use std::fs::read_to_string;
use std::path::PathBuf;

use crate::errors::Result;

mod package;

use package::{Package, RootPath};


pub fn verify(path: PathBuf) {
    match read_to_string(path) {
        Ok(data) => match toml::from_str::<Package>(&data) {
            Ok(package) => {
                if !package.is_valid() {
                    return;
                }
                eprintln!("Manifest: OK");
            }
            Err(e) => eprintln!("Error: {:#?}", e),
        },
        Err(e) => eprintln!("Failed {:#?}", e),
    }
}

pub fn install_packages(requirements_file: Option<PathBuf>, path: Option<String>, update: bool) {
    let packages = match (requirements_file, path) {
        (Some(req_file), _) => {
            let package_list = read_requirements_file(req_file);
            get_packages(package_list)
        }
        (_, Some(p)) => get_packages(vec![p.into()]),
        _ => {
            unreachable!("no requirements file or path to a package supplied");
        }
    };

    for (root, package) in packages {
        if package.is_valid() {
            match update {
                true => package.update(root, format!("./pack/{}", package.safe_name()).into()),
                false => package.install(root, format!("./pack/{}", package.safe_name()).into()),
            }
        } else {
            eprintln!("Invalid manifest: {:?}", package.name());
        }
    }
}

fn read_requirements_file(path: PathBuf) -> Vec<String> {
    let data = match read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed: {:?}", e);
            return Vec::new();
        }
    };
    data.trim().split('\n').map(|l| l.trim().to_string()).collect()
}

fn get_packages(package_paths: Vec<String>) -> Vec<(RootPath, Package)> {
    let mut packages = Vec::new();

    for line in package_paths {
        let line = line.trim();
        if line.starts_with("#") {
            continue
        }
        if line.starts_with("/") {
            // local file
            let path: PathBuf = line.into();
            match local_package(path.join("manifest.toml")) {
                Ok(package) => packages.push((RootPath::Local(path), package)),
                Err(e) => eprintln!("failed to load (local) package: {:?}", e),
            }
        } else {
            // remote file full url
            let mut url = match line.starts_with("https://") {
                true => line.to_string(),
                false => to_github_raw(line),
            };

            if !url.ends_with("/") {
                url.push('/');
            }

            match remote_package(&format!("{}manifest.toml", url)) {
                Ok(package) => packages.push((RootPath::Remote(url), package)),
                Err(_e) => eprintln!("failed to load (remote) package. Invalid or missing manifest file"),
            }
        }
    }

    packages
}

fn local_package(path: PathBuf) -> Result<Package> {
    let data = read_to_string(path)?;
    Ok(toml::from_str::<Package>(&data)?)
}

fn remote_package(url: &str) -> Result<Package> {
    let data = reqwest::blocking::get(url)?.text()?;
    Ok(toml::from_str::<Package>(&data)?)
}

fn to_github_raw(path: &str) -> String {
    let parts = path.split("/").collect::<Vec<_>>();
    if parts.len() != 2 {
        eprintln!("{:?}", "invalid path");
    }
    format!("https://raw.githubusercontent.com/{}/godot-packages/master/{}/", parts[0], parts[1])
}
