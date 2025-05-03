use std::process::Command;

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo::rerun-if-changed=slang_shaders/*");

    Command::new("rm")
        .arg("./slang_shaders/bin/render.spv")
        .output();

    
    let mut command = Command::new("slangc");
    command
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
        .arg("compute");
    
    if cfg!(debug_assertions) {
        command.arg("-g3")
            .arg("-O0") 
    } else {
        command.arg("-g0")
            .arg("-O3")
    };

    let res = command.output();
    if res.is_err() {
        panic!("{:?}", res);
    } else {
        let output = res.unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        if output.status.success() {
            warn!("Compiled successfully render.slang. \n {} {}", stdout, stderr);
        } else {
            panic!("Compile failed render.slang: {} {}", stdout, stderr);
        }
    }
}
