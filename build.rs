use std::{fs, process::Command, time::{SystemTime}};

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
    println!("cargo:rerun-if-changed=shaders/");
    println!("cargo:rerun-if-changed=shader_constants.env");
    warn_lines(format!("Ran at: {:?}", SystemTime::now()));

    let constants = get_compile_time_constants();
    for (k, v) in constants.iter() {
        println!("cargo:rustc-env={k}={v}");
    }

    compile_shader("_trace_scene", "main", &constants);
    compile_shader("_blit", "main", &constants);
    compile_shader("_temporal_denoise", "main", &constants);
    compile_shader("_a_tours_filter", "main", &constants);
    compile_shader("mesh", "vertex", &constants);
    compile_shader("mesh", "fragment", &constants);
    compile_shader("_gi_probe_update", "main", &constants);
}

fn get_compile_time_constants() -> Vec<(String, String)> {
    let contents = fs::read_to_string("shader_constants.env").unwrap();

    contents.lines()
        .filter_map(|s| s.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

fn compile_shader(name: &str, entry: &str, constants: &[(String, String)]) {
    let source_path = format!("./shaders/{name}.slang");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let spv_path = format!("{out_dir}/{name}_{entry}.spv");
        
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
        .arg(entry);

    for (k, v) in constants {
        command.arg(format!("-D{k}={v}"));
    }
    
    if cfg!(debug_assertions) {
        command.arg("-g3").arg("-O0") 
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
