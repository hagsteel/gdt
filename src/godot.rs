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

pub fn init(name: String, run_cargo_init: bool) {
    // Create directory
    let project_file_path = PathBuf::from(&name);
    let _ = create_dir_all(&project_file_path);

    // Create project template
    let template = TEMPLATE.replace("{name}", &name);
    let template_file = project_file_path.join(PROJECT_TEMPLATE_NAME);
    create_file(template_file, &template);
    eprintln!("Created godot project file");

    // Create project requirements
    let requirements_file = project_file_path.join(REQUIREMENTS_FILE);
    create_file(requirements_file, REQUIREMENTS);
    eprintln!("Created requirements file");

    // Cargo init
    if run_cargo_init {
        let res = cargo_init(name);
        if let Err(e) = res {
            eprintln!("Failed to init Rust project: {:?}", e);
        }
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

    let full_path = current_dir().unwrap().join(name);
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
