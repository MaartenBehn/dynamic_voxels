#ifndef _CGS_GLSL_
#define _CGS_GLSL_

#include "binding.glsl"
#include "ray.glsl"
#include "easing.glsl"
#include "mat_helper.glsl"
#include "debug.glsl"

#define CGS_TYPE_BOX 0
#define CGS_TYPE_SPHERE 1
#define CGS_TYPE_CAPSULE 2

struct CGSObject {
    mat4 transform;
    vec3 data;
    uint type;
};

struct CGSChild {
    uint pointer;
    uint material;
    uint type;
};

CGSObject get_csg_tree_object(uint index) {
    mat4 transform = mat4(
        uintBitsToFloat(CSG_TREE[index    ]), uintBitsToFloat(CSG_TREE[index + 1]), uintBitsToFloat(CSG_TREE[index +  2]), uintBitsToFloat(CSG_TREE[index +  3]),
        uintBitsToFloat(CSG_TREE[index + 4]), uintBitsToFloat(CSG_TREE[index + 5]), uintBitsToFloat(CSG_TREE[index +  6]), uintBitsToFloat(CSG_TREE[index +  7]),
        uintBitsToFloat(CSG_TREE[index + 8]), uintBitsToFloat(CSG_TREE[index + 9]), uintBitsToFloat(CSG_TREE[index + 10]), uintBitsToFloat(CSG_TREE[index + 11]),
        0.0, 0.0, 0.0, 1.0
    );
    vec3 data = vec3(uintBitsToFloat(CSG_TREE[index + 12]), uintBitsToFloat(CSG_TREE[index + 13]), uintBitsToFloat(CSG_TREE[index + 14]));
    uint type = CSG_TREE[index + 15];

    return CGSObject(transform, data, type);
}

CGSChild get_csg_tree_child(uint index) {
    uint data = CSG_TREE[index];
    uint pointer = data >> 16;         // 16 Bit
    uint material = data & uint(63);   //  6 Bit
    uint type = (data >> 6) & uint(3); //  2 Bit

    return CGSChild(pointer, material, type);
}

CGSObject get_test_box(float time, vec3 pos) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 2.0;

    mat4 rot_mat = mat4_rotate_xyz(vec3(
        time_mod_rot(time, 0.2),
        time_mod_rot(time, 1.0),
        time_mod_rot(time, 0.4)));

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 1.5)) * rot_mat * mat4_pos(pos));

    return CGSObject(
        mat,
        vec3(1.0),
        CGS_TYPE_BOX
    );
}

CGSObject get_test_sphere(float time, vec3 pos) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 3.0)) * mat4_pos(pos));

    return CGSObject(
    mat,
    vec3(0.0),
    CGS_TYPE_SPHERE
    );
}

vec4 ray_hits_cgs_object(Ray ray, CGSObject object) {
    Ray model_space_ray = ray_to_model_space(ray, object.transform);

    float t_min;
    float t_max;
    bool hit;
    if (object.type == CGS_TYPE_BOX) {
       hit = ray_aabb_intersect(model_space_ray, vec3(-0.5), vec3(0.5), t_min, t_max);
    } else if (object.type == CGS_TYPE_SPHERE) {
        hit = ray_sphere_intersect(model_space_ray, t_min, t_max);
    }

    if (hit) {
        return vec4(get_debug_color_gradient_from_float(t_min / 25.0), 1.0);
    } else {
        return vec4(0.0);
    }
}


#endif // _CGS_GLSL_