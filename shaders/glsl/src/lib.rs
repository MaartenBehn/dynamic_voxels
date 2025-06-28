use glsl_compiler::glsl;

pub fn trace_ray() -> (&'static [u8], &'static [&'static str]) {
    glsl! {type = Compute, file = "shaders/glsl/glsl/trace_ray.glsl"}
}

pub fn trace_ray_profile() -> (&'static [u8], &'static [&'static str]) {
    glsl! {type = Compute, profile, file = "shaders/glsl/glsl/trace_ray.glsl"}
}
