use gdnative::*;


fn init(handle: init::InitHandle) {
    // handle.add_class::<>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();


#[no_mangle]
pub extern fn run_tests() -> sys::godot_variant {
    let status = false;

    eprintln!("Running tests: [add your tests here]");

    gdnative::Variant::from_bool(status).forget()
}
