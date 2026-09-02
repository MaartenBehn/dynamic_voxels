use octa_force::build::{ReflectedSampler, codegen};
use octa_force::build::slang_reflection::{extract_samplers};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

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
    warn_lines(format!("OUT_DIR: {:?}", std::env::var("OUT_DIR").unwrap()));

    let constants = get_compile_time_constants();
    for (k, v) in constants.iter() {
        println!("cargo:rustc-env={k}={v}");
    }

    let mut all_samplers: HashMap<String, ReflectedSampler> = HashMap::new();

    compile_shader("_trace_scene", "main", &constants, &mut all_samplers);
    compile_shader("_blit", "main", &constants, &mut all_samplers);
    compile_shader("_temporal_denoise", "main", &constants, &mut all_samplers);
    compile_shader("_a_tours_filter", "main", &constants, &mut all_samplers);
    compile_shader("mesh", "vertex", &constants, &mut all_samplers);
    compile_shader("mesh", "fragment", &constants, &mut all_samplers);
    compile_shader("_gi_probe_update", "main", &constants, &mut all_samplers);

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let gen_rs_path = out_dir.join("generated_samplers.rs");
    
    let samplers_vec: Vec<_> = all_samplers.into_values().collect();
    codegen::generate_sampler_bindings(&samplers_vec, &gen_rs_path);
}

fn get_compile_time_constants() -> Vec<(String, String)> {
    let contents = fs::read_to_string("shader_constants.env").unwrap();

    contents
        .lines()
        .filter_map(|s| s.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

fn compile_shader(
    name: &str,
    entry: &str,
    constants: &[(String, String)],
    sampler_accumulator: &mut HashMap<String, ReflectedSampler>,
) {
    let source_path = format!("./shaders/{name}.slang");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let spv_path = format!("{out_dir}/{name}_{entry}.spv");
    let json_reflection_path = format!("{out_dir}/{name}_{entry}_reflection.json");

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
        .arg(entry)
        .arg("-reflection-json")
        .arg(&json_reflection_path);

    for (k, v) in constants {
        command.arg(format!("-D{k}={v}"));
    }

    if cfg!(debug_assertions) {
        command.arg("-g3").arg("-O0");
    } else {
        command.arg("-g3").arg("-O0");
    }

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

            if let Ok(json_text) = fs::read_to_string(&json_reflection_path) {
                let samplers = extract_samplers(&json_text);
                for s in samplers {


                    let key = format!("{}_{}", s.set, s.binding);
                    sampler_accumulator.insert(key, s);
                }
            }
        } else {
            panic!("Compile failed {}.slang: {} {}", name, stdout, stderr);
        }
    }
}
