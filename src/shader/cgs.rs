use glsl_compiler::glsl;

#[allow(unused)]
pub fn cgs_shader() {
    glsl!{type = Include, name = "shaders", code = {
        #ifndef _CGS_GLSL_
        #define _CGS_GLSL_

        #define CGS_TYPE_BOX 0
        #define CGS_TYPE_SPHERE 1
        #define CGS_TYPE_CAPSULE 2

        struct CGS {
            int type;
            int child_pointer;
            int child_count;
            mat4 transform;
            vec3 data;
        };

        mat4 mat_identity() {
            return mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );
        }

        CGS[1] get_cgs_tree() {
            mat4 mat = mat_identity();
            return CGS[1](CGS(CGS_TYPE_BOX, 1, 0, mat, vec3(1, 1, 1)));
        }

        #endif // _CGS_GLSL_
    }};
}