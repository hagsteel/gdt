use std::io::Write;
use std::fs::File;

const PROJECT_TEMPLATE_NAME: &'static str = "project.godot";
const TEMPLATE: &'static str = r#"[application]

config/name={name}

[display]

window/size/width=1280
window/size/height=720
"#;

pub fn init(name: String) {
    let template = TEMPLATE.replace("{name}", &name);
    let mut file = match File::create(PROJECT_TEMPLATE_NAME) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {:?}", e);
            return
        }
    };
    let _ = file.write(template.as_bytes());
}
