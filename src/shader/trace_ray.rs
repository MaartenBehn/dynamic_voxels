use glsl_compiler::glsl;

pub fn trace_ray_shader() -> &'static [u8] {
    let bin: &[u8] = glsl!{type = Compute, code = {
        #version 450 core

        #include "./src/shaders/ray.rs-shaders"
        #include "./src/shaders/cgs.rs-shaders"

        #define MAX_RAY_STEPS 50
        #define EPSILON 0.0001
        #define TO_1D(pos, size) ((pos.z * size * size) + (pos.y * size) + pos.x)

        layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

        layout(binding = 0, rgba8) uniform writeonly image2D img;

        layout(binding = 1) uniform UniformBufferObject {
            vec4 pos; // w is screen_size_x
            vec4 dir; // w is screen_size_y
        } render_buffer;

        #define POS render_buffer.pos.xyz
        #define DIR render_buffer.dir.xyz
        #define RES_X render_buffer.pos.w
        #define RES_Y render_buffer.dir.w
        #define RES vec2(RES_X, RES_Y)

        vec4 render_cgs_tree() {
            CGS[1] tree = get_cgs_tree();

            int i = 0;
            while(i < tree.length()) {
                CGS cgs = tree[i];
                if cgs.type == CGS_TYPE_BOX {

                }

                i++;
            }

            return vec4(0.0, 0.0, 0.0, 1.0);
        }

        void main () {
            Ray ray = init_ray(POS, DIR, gl_GlobalInvocationID.xy, RES);

            vec4 color = vec4(ray.dir, 1);

            imageStore(img, ivec2(gl_GlobalInvocationID.xy), color);
        }
    }};

    bin
}