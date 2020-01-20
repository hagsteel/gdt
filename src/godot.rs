use std::io::Write;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;

use crate::errors::Result;

const PROJECT_TEMPLATE_NAME: &'static str = "project.godot";
const TEMPLATE: &'static str = r#"; Engine configuration file.
; Generated with GUT

config_version=4

_global_script_classes=[  ]
_global_script_class_icons={

}

[application]

config/name="{name}"

[rendering]

environment/default_environment="res://default_env.tres"


[display]

window/size/width=1280
window/size/height=720
"#;

const REQUIREMENTS_FILE: &'static str = "requirements.txt";
const REQUIREMENTS: &'static str = "hagsteel/simple-sprite";

pub fn init(name: String) {
    let godot_project_file_path = PathBuf::from(&name).join("godot");
    let rust_project_file_path = PathBuf::from(&name).join("rust");

    // Create directory
    let _ = create_dir_all(&godot_project_file_path);
    let _ = create_dir_all(&rust_project_file_path);

    // Create project template
    let template = TEMPLATE.replace("{name}", &name);
    let template_file = godot_project_file_path.join(PROJECT_TEMPLATE_NAME);
    create_file(template_file, &template);
    eprintln!("Created godot project file");

    // Create project requirements
    let requirements_file = godot_project_file_path.join(REQUIREMENTS_FILE);
    create_file(requirements_file, REQUIREMENTS);
    eprintln!("Created requirements file");

    // Cargo init
    let res = cargo_init(name);
    if let Err(e) = res {
        eprintln!("Failed to init Rust project: {:?}", e);
    }
}

fn create_file(path: PathBuf, content: &str) {
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {:?}", e);
            return
        }
    };
    let _ = file.write(content.as_bytes());
}

// -----------------------------------------------------------------------------
//     - Rust's cargo init -
// -----------------------------------------------------------------------------
fn cargo_init(name: String) -> Result<()> {
    use std::env::current_dir;
    use std::fs::read_to_string;
    use cargo::ops;
    use cargo::util::config;

    let full_path = current_dir().unwrap().join(&name).join("rust");
    let cargo_path = full_path.join("Cargo.toml");
    if cargo_path.exists() {
        eprintln!("Rust project already exists");
        return Ok(())
    }

    // Cargo init options
    let opts = ops::NewOptions::new(
        None,
        false,
        true,
        full_path.clone(),
        None,
        None,
        None
    ).unwrap();

    // Cargo config
    let config = config::Config::default()?;

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

    let cargo_toml = lines.join("\n");
    let mut cargo_file = File::create(cargo_path)?;
    cargo_file.write_all(cargo_toml.as_bytes())?;

    Ok(())
}
