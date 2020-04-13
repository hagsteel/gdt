use std::collections::HashMap;
use std::fs::{
    create_dir_all, metadata, read_to_string, remove_file, rename, set_permissions, File,
    read_dir, ReadDir
};
use std::io::Write;
use std::path::PathBuf;

use dirs::home_dir;
use handlebars;

use crate::errors::Result;

fn create_project_files(project_root: &PathBuf, template_root: &PathBuf, files: ReadDir, context: &HashMap<&str, &str>) {
    let hb = handlebars::Handlebars::new();

    for file in files {
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                eprintln!("failed to get file: {:?}", e);
                continue
            }
        };
        let file_path = file.path();
        let child = match file_path.strip_prefix(&template_root) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("failed to strip prefix: {:?}", e);
                continue
            }
        };

        if file.path().is_dir() {
            // Create directory
            if let Ok(_) = create_dir_all(project_root.join(child)) {
                // project_root.join(child);
                let children = match read_dir(&template_root.join(child)) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("failed to read sub directory: {:?}", e);
                        continue
                    }
                };
                create_project_files(&project_root, &template_root, children, context);
            }
        } else {
            let src = &file.path();
            let perms = match metadata(&src) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("failed to get permissions for {:?}: {:?}", src, e);
                    continue
                }
            }.permissions();

            let child_str = match child.to_str() {
                Some(s) => match hb.render_template(s, &context) {
                    Ok(tpl) => tpl,
                    Err(e) => {
                        eprintln!("failed to render template: {:?}", e);
                        continue
                    }
                }
                None => {
                    continue
                }
            };
            let dst = project_root.join(child_str);

            let template = match read_to_string(src) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("failed to read {:?} : {:?}", src, e);
                    continue;
                }
            };
            let rendered = match hb.render_template(&template, &context) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("failed to render {} : {:?}", template, e);
                    continue
                }
            };
            create_file(dst.to_path_buf(), &rendered);

            if let Err(e) = set_permissions(dst.to_path_buf(), perms) {
                eprintln!("failed to set permissions on {:?} : {:?}", dst, e);
            }
        }
    }

}

pub fn init(name: String) {
    let project_root = PathBuf::from(&name);
    let template_root = home_dir().unwrap().join(".config/gdt/templates/");
    let files = read_dir(&template_root).unwrap();

    let rust_project_file_path = project_root.join("rust");

    let mut context: HashMap<&str, &str> = HashMap::new();
    context.insert("name", &name);

    match cargo_init(&name) {
        Ok(_) => {
            let _ = rename(
                &rust_project_file_path.join(".git"),
                project_root.join(".git"),
            );
            let _ = remove_file(&rust_project_file_path.join(".gitignore"));
        }
        Err(e) => {
            eprintln!("Failed to init Rust project: {:?}", e);
        }
    }

    create_project_files(&project_root, &template_root, files, &context);
}

fn create_file(path: PathBuf, content: &str) {
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {:?}", e);
            return;
        }
    };
    let _ = file.write(content.as_bytes());
}

// -----------------------------------------------------------------------------
//     - Rust's cargo init -
// -----------------------------------------------------------------------------
fn cargo_init(name: &str) -> Result<()> {
    use cargo::ops;
    use cargo::util::config;
    use std::env::current_dir;

    let full_path = current_dir().unwrap().join(&name).join("rust");
    let cargo_path = full_path.join("Cargo.toml");
    if cargo_path.exists() {
        eprintln!("Rust project already exists");
        return Ok(());
    }

    // Cargo init options
    let opts =
        ops::NewOptions::new(None, false, true, full_path.clone(), None, None, None).unwrap();

    // Cargo config
    let config = config::Config::default().unwrap();

    // Init project
    let _ = ops::init(&opts, &config);

    // Set the crate type
    let cargo_toml = read_to_string(&cargo_path)?.trim().to_string();
    let mut lines = cargo_toml.split("\n").collect::<Vec<_>>();

    // Set the name to the project name
    let name_entry = format!("name = \"{}\"", name);
    lines.remove(0);
    lines.remove(0);
    lines.insert(0, &name_entry);
    lines.insert(0, "[package]");

    // Set the crate type
    lines.pop(); // remove the [dependencies] lines
    lines.push("[lib]");
    lines.push("crate-type = [\"dylib\"]");
    lines.push("");
    lines.push("[dependencies]");
    lines.push("gdnative = \"0.8.0\"");
    lines.push("gdextras = { path = \"../../gdextras\"} ");

    let cargo_toml = lines.join("\n");
    let mut cargo_file = File::create(cargo_path)?;
    cargo_file.write_all(cargo_toml.as_bytes())?;

    Ok(())
}
