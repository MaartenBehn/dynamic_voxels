use std::process::Command;

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn warn_lines(text: String) {
    for line in text.lines() {
        warn!("{}", line)
    }
}

fn main() {
    println!("cargo::rerun-if-changed=shaders/*");
    
    compile_shader("_trace_tree64");
    compile_shader("_trace_scene");
    compile_shader("_blit");
    compile_shader("_temporal_denoise");
    compile_shader("_a_tours_filter");
}

fn compile_shader(name: &str) {
    let source_path = format!("./shaders/{name}.slang");
    let spv_path = format!("./shaders/bin/{name}.spv");

    Command::new("rm")
        .arg(&spv_path)
        .output();

    
    let mut command = Command::new("slangc");
    command
        .arg(&source_path)
        .arg("-profile")
        .arg("glsl_450")
        .arg("-target")
        .arg("spirv")
        .arg("-o")
        .arg(&spv_path)
        .arg("-entry")
        .arg("compute_main");
    
    if cfg!(debug_assertions) {
        command.arg("-g3")
            .arg("-O0") 
    } else {
        command.arg("-g0").arg("-O3")
        //command.arg("-g3").arg("-O0")
    };

    let res = command.output();
    if res.is_err() {
        panic!("{:?}", res);
    } else {
        let output = res.unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        if output.status.success() {
            warn!("Compiled successfully {}.slang.", name);
            warn_lines(stdout);
            warn_lines(stderr);
        } else {
            panic!("Compile failed {}.slang: {} {}", name, stdout, stderr);
        }
    }

}
