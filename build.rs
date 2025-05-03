use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=slang_shaders/*");

    Command::new("slangc")
        .arg("./slang_shaders/render.slang")
        .arg("-profile")
        .arg("glsl_450")
        .arg("-target")
        .arg("spirv")
        .arg("-o")
        .arg("./slang_shaders/bin/render.spv")
        .arg("-entry")
        .arg("compute_main")
        .arg("-stage")
        .arg("compute")
        .spawn()
        .unwrap();
}
