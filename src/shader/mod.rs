use glsl_compiler::glsl;

pub fn trace_ray_shader() -> &'static [u8] {
    let bin: &[u8] = glsl!{type = Compute, file = "shaders/trace_ray.comp"};
    bin
}