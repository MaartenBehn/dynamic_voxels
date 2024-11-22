#ifndef _CGS_GLSL_
#define _CGS_GLSL_

#include "ray.glsl"
#include "easing.glsl"
#include "mat_helper.glsl"

#define CGS_TYPE_BOX 0
#define CGS_TYPE_SPHERE 1
#define CGS_TYPE_CAPSULE 2

struct CGSObject {
    mat4 transform;
    uint type;
    vec3 data;
};

struct CGSChild {
    uint pointer;
    uint material;
    uint type;
};


CGSObject get_test_box(float time) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 rot_mat = mat4_rotate_xyz(vec3(
        time_mod_rot(time, 0.2),
        time_mod_rot(time, 1.0),
        time_mod_rot(time, 0.4)));

    mat4 mat = inverse(rot_mat * mat4_scale(vec3(scale)));

    return CGSObject(
        mat,
        CGS_TYPE_BOX,
        vec3(1.0)
    );
}

CGSObject get_test_sphere(float time) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 mat = inverse(mat4_scale(vec3(scale)));

    return CGSObject(
    mat,
    CGS_TYPE_SPHERE,
    vec3(0.0)
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
        hit = ray_sphere_intersect(model_space_ray, 1.0, vec3(0.0), t_min, t_max);
    }

    if (hit) {
        return vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4(0.0);
    }
}


#endif // _CGS_GLSL_