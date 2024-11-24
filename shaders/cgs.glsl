#ifndef _CGS_GLSL_
#define _CGS_GLSL_

#include "binding.glsl"
#include "ray.glsl"
#include "easing.glsl"
#include "mat_helper.glsl"
#include "debug.glsl"

#define CGS_GEO_TYPE_BOX 0
#define CGS_GEO_TYPE_SPHERE 1
#define CGS_GEO_TYPE_CAPSULE 2

#define CGS_CHILD_TYPE_GEO 0
#define CGS_CHILD_TYPE_UNION 1
#define CGS_CHILD_TYPE_REMOVE 2
#define CGS_CHILD_TYPE_INTERSECT 3

#define MAX_CGS_TREE_DEPTH 10
#define MAX_CGS_RENDER_ITERATIONS 10

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
        uintBitsToFloat(CSG_TREE[index    ]), uintBitsToFloat(CSG_TREE[index + 4]), uintBitsToFloat(CSG_TREE[index +  8]), uintBitsToFloat(CSG_TREE[index +  12]),
        uintBitsToFloat(CSG_TREE[index + 1]), uintBitsToFloat(CSG_TREE[index + 5]), uintBitsToFloat(CSG_TREE[index +  9]), uintBitsToFloat(CSG_TREE[index +  13]),
        uintBitsToFloat(CSG_TREE[index + 2]), uintBitsToFloat(CSG_TREE[index + 6]), uintBitsToFloat(CSG_TREE[index + 10]), uintBitsToFloat(CSG_TREE[index +  14]),
        0.0, 0.0, 0.0, 1.0
    );
    vec3 data = vec3(uintBitsToFloat(CSG_TREE[index + 3]), uintBitsToFloat(CSG_TREE[index + 7]), uintBitsToFloat(CSG_TREE[index + 11]));
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
        CGS_GEO_TYPE_BOX
    );
}

CGSObject get_test_sphere(float time, vec3 pos) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 3.0)) * mat4_pos(pos));

    return CGSObject(
    mat,
    vec3(0.0),
    CGS_GEO_TYPE_SPHERE
    );
}

bool ray_hits_cgs_object(Ray ray, CGSObject object, out float t_min, out float t_max) {
    Ray model_space_ray = ray_to_model_space(ray, object.transform);

    if (object.type == CGS_GEO_TYPE_BOX) {
       return ray_aabb_intersect(model_space_ray, vec3(-0.5), vec3(0.5), t_min, t_max);
    } else if (object.type == CGS_GEO_TYPE_SPHERE) {
        return ray_sphere_intersect(model_space_ray, t_min, t_max);
    }

    return false;
}

void cgs_operation(float t_min_1, float t_max_1, float t_min_2, float t_max_2, uint operation, out float t_min, out float t_max) {
    if (operation == CGS_CHILD_TYPE_UNION) {
        t_min = min(t_min_1, t_min_2);
        t_max = max(t_max_1, t_max_2);

    } else if (operation == CGS_CHILD_TYPE_REMOVE) {
        if (t_min_2 < t_min_1 && t_min_1 < t_max_2) {
            t_min = t_max_2;
        } else {
            t_min = t_min_1;
        }

        if (t_min_2 < t_max_1 && t_max_1 < t_max_2) {
            t_max = t_min_2;
        } else {
            t_max = t_max_1;
        }

    } else if (operation == CGS_CHILD_TYPE_INTERSECT) {

    }
}


vec4 render_cgs_tree(Ray ray) {
    uint stack_len = 0;
    uint stack[MAX_CGS_TREE_DEPTH];
    uint operation_stack[MAX_CGS_TREE_DEPTH + 1];
    float t_min_1_stack[MAX_CGS_TREE_DEPTH + 1];
    float t_max_1_stack[MAX_CGS_TREE_DEPTH + 1];
    operation_stack[0] = CGS_CHILD_TYPE_UNION;

    const float pos_infinity = uintBitsToFloat(0x7F800000);
    const float neg_infinity = uintBitsToFloat(0xFF800000);

    for (uint i = 0; i < MAX_CGS_TREE_DEPTH + 1; i++) {
        t_min_1_stack[i] = pos_infinity;
        t_max_1_stack[i] = neg_infinity;
    }

    bool is_left = false;
    bool left = true;
    bool perform = false;

    uint current = 0;
    CGSChild child;
    float t_min_2 = pos_infinity;
    float t_max_2 = neg_infinity;
    float t_min = pos_infinity;
    float t_max = neg_infinity;

    bool hit = false;
    uint i = 0;
    while (i < MAX_CGS_RENDER_ITERATIONS) {
        i++;

        if (perform) {
            uint operation = operation_stack[stack_len];
            float t_min_1 = t_min_1_stack[stack_len];
            float t_max_1 = t_max_1_stack[stack_len];

            t_min = pos_infinity;
            t_max = neg_infinity;
            cgs_operation(t_min_1, t_max_1, t_min_2, t_max_2, operation, t_min, t_max);

            stack_len--;
            if (is_left) {

                t_min_1_stack[stack_len] = t_min;
                t_max_1_stack[stack_len] = t_max;

                perform = false;
                left = false;

                if (stack_len == 0) {
                    is_left = false;
                }
            } else {

                t_min_2 = t_min;
                t_max_2 = t_max;

                if (stack_len == 0) {
                    break;
                }
            }

            current = stack[stack_len];
            continue;
        }

        if (left) {
            child = get_csg_tree_child(current);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                stack[stack_len] = current;

                stack_len++;
                operation_stack[stack_len] = child.type;

                current = child.pointer;

                is_left = true;
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);
                bool hit = ray_hits_cgs_object(ray, object, t_min_1_stack[stack_len], t_max_1_stack[stack_len]);
                if (!hit) {
                    t_min_1_stack[stack_len] = pos_infinity;
                    t_max_1_stack[stack_len] = neg_infinity;
                }

                left = false;
            }
        } else {
            child = get_csg_tree_child(current + 1);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                stack[stack_len] = current;
                stack_len++;

                current = child.pointer;
                is_left = false;
                left = true;
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);
                bool hit = ray_hits_cgs_object(ray, object, t_min_2, t_max_2);
                if (!hit) {
                    t_min_2 = pos_infinity;
                    t_max_2 = neg_infinity;
                }

                perform = true;
            }
        }
    }

    if (t_max > 0 && t_max > t_min) {
        //return vec4(get_debug_color_gradient_from_float(float(i) / 10.0), 1.0);
        return vec4(get_debug_color_gradient_from_float(t_min / 25.0), 1.0);
    }

    return vec4(0.0);
}


#endif // _CGS_GLSL_