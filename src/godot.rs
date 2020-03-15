use std::io::Write;
use std::fs::{File, create_dir_all, set_permissions, metadata, remove_file, rename};
use std::os::unix::fs::PermissionsExt;
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
    let project_root = PathBuf::from(&name);
    let godot_project_file_path = project_root.join("godot");
    let rust_project_file_path = project_root.join("rust");

    // Create directories
    let _ = create_dir_all(&godot_project_file_path);
    let _ = create_dir_all(&godot_project_file_path.join("lib"));
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

    // Cargo init (and move .git dir)
    match cargo_init(&name) {
        Ok(_) => { 
            let _ = rename(&rust_project_file_path.join(".git"), project_root.join(".git")); 
            let _ = remove_file(&rust_project_file_path.join(".gitignore")); 
            let _ = create_git_ignore(project_root.join(".gitignore"));
        }
        Err(e) => {
            eprintln!("Failed to init Rust project: {:?}", e);
        }
    }

    // Create gdnlib file
    if let Err(e) = create_native_lib_file(&name, godot_project_file_path.join(format!("lib{}.gdnlib", name))) {
        eprintln!("Failed to create gdnlib file: {:?}", e);
    }

    // Create default env file
    if let Err(e) = create_env_file(godot_project_file_path.join("default_env.tres")) {
        eprintln!("Failed to create gdnlib file: {:?}", e);
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
fn cargo_init(name: &str) -> Result<()> {
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
    lines.push("gdnative = \"0.8.0\" }");
    lines.push("gdextras = { path = \"../../gdextras\"} ");

    let cargo_toml = lines.join("\n");
    let mut cargo_file = File::create(cargo_path)?;
    cargo_file.write_all(cargo_toml.as_bytes())?;

    let build_file_path = full_path.join("build.sh");
    create_build_script(&name, build_file_path.into());

    let watch_file_path = full_path.join("watch.sh");
    create_watch_script(&name, watch_file_path.into());

    Ok(())
}

fn create_build_script(project_name: &str, path: PathBuf) -> Result<()> {
    let file_content = format!(r#"#!/bin/sh
tmux renamew -t $TWINDOW building...
clear
if cargo build --release; then
    cp target/release/lib{name}.so ../godot/lib/lib{name}.so
    tmux renamew -t $TWINDOW Ok
    buildstreak success
    buildstreak tmux > /tmp/buildstreak
    mplayer ~/Documents/ok.wav 1>&- 2>&-
else
    tmux renamew -t $TWINDOW Err...
    buildstreak fail
    buildstreak tmux > /tmp/buildstreak
    mplayer ~/Documents/err.wav 1>&- 2>&-
    exit 1
fi
"#, name=project_name);

    let mut build_file = File::create(&path)?;
    build_file.write_all(file_content.as_bytes())?;
    let mut perms = metadata(&path)?.permissions();
    perms.set_mode(0o777);
    set_permissions(path, perms);

    Ok(())
}

fn create_watch_script(project_name: &str, path: PathBuf) -> Result<()> {
    let file_content = r#"#!/bin/sh
export TWINDOW="$(tmux display-message -p '#I')"
cargo watch -s './build.sh' -w src/ -w ../../gdextras/ "#;

    let mut watch_file = File::create(&path)?;
    watch_file.write_all(file_content.as_bytes())?;
    let mut perms = metadata(&path)?.permissions();
    perms.set_mode(0o777);
    set_permissions(path, perms);

    Ok(())
}

fn create_native_lib_file(project_name: &str, path: PathBuf) -> Result<()> {
    let file_content = format!(r#"[entry]

X11.64="res://lib/lib{name}.so"

[dependencies]

X11.64=[  ]

[general]

singleton=false
load_once=true
symbol_prefix="godot_"
reloadable=true"#, name=project_name);

    let mut gdnlib_file = File::create(&path)?;
    gdnlib_file.write_all(file_content.as_bytes())?;

    Ok(())
}

fn create_git_ignore(path: PathBuf) -> Result<()> {
    let file_content = r#"/rust/target
**/*.rs.bk
Cargo.lock"#;

    let mut git_ignore_file = File::create(&path)?;
    git_ignore_file.write_all(file_content.as_bytes())?;

    Ok(())
}

fn create_env_file(path: PathBuf) -> Result<()> {
    let file_content = r#"[gd_resource type="Environment" load_steps=2 format=2]
[sub_resource type="ProceduralSky" id=1]
[resource]
background_mode = 2
background_sky = SubResource( 1 )"#;

    let mut env_file = File::create(&path)?;
    env_file.write_all(file_content.as_bytes())?;

    Ok(())
}
