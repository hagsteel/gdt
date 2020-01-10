use std::io::Write;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;

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
