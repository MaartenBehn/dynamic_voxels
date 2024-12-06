#ifndef _CGS_GLSL_
#define _CGS_GLSL_

#include "binding.glsl"
#include "ray.glsl"
#include "easing.glsl"
#include "mat_helper.glsl"
#include "debug.glsl"
#include "dda.glsl"

#define ONLY_RENDER_AABB false
#define ONLY_RENDER_CGS false
#define USE_AABB true

#define CGS_GEO_TYPE_BOX 0
#define CGS_GEO_TYPE_SPHERE 1
#define CGS_GEO_TYPE_CAPSULE 2

#define CGS_CHILD_TYPE_GEO 0
#define CGS_CHILD_TYPE_UNION 1
#define CGS_CHILD_TYPE_REMOVE 2
#define CGS_CHILD_TYPE_INTERSECT 3

#define MAX_CGS_TREE_DEPTH 4
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

struct AABB {
    vec3 min;
    vec3 max;
};


CGSObject get_csg_tree_object(uint index) {
    PROFILE("get_csg_tree_object");

    mat4 transform = mat4(
        uintBitsToFloat(CSG_TREE[index     + 6]), uintBitsToFloat(CSG_TREE[index + 4 + 6]), uintBitsToFloat(CSG_TREE[index +  8 + 6]), uintBitsToFloat(CSG_TREE[index +  12 + 6]),
        uintBitsToFloat(CSG_TREE[index + 1 + 6]), uintBitsToFloat(CSG_TREE[index + 5 + 6]), uintBitsToFloat(CSG_TREE[index +  9 + 6]), uintBitsToFloat(CSG_TREE[index +  13 + 6]),
        uintBitsToFloat(CSG_TREE[index + 2 + 6]), uintBitsToFloat(CSG_TREE[index + 6 + 6]), uintBitsToFloat(CSG_TREE[index + 10 + 6]), uintBitsToFloat(CSG_TREE[index +  14 + 6]),
        0.0, 0.0, 0.0, 1.0
    );
    vec3 data = vec3(uintBitsToFloat(CSG_TREE[index + 3 + 6]), uintBitsToFloat(CSG_TREE[index + 7 + 6]), uintBitsToFloat(CSG_TREE[index + 11 + 6]));
    uint type = CSG_TREE[index + 15 + 6];

    return CGSObject(transform, data, type);
}

CGSChild get_csg_tree_child(uint index) {
    PROFILE("get_csg_tree_child");

    uint data = CSG_TREE[index + 6];
    uint pointer = data >> 16;         // 16 Bit
    uint material = data & uint(63);   //  6 Bit
    uint type = (data >> 6) & uint(3); //  2 Bit

    return CGSChild(pointer, material, type);
}

AABB get_aabb(uint index) {
    PROFILE("get_aabb");

    vec3 min = vec3(uintBitsToFloat(CSG_TREE[index    ]), uintBitsToFloat(CSG_TREE[index + 1]), uintBitsToFloat(CSG_TREE[index + 2]));
    vec3 max = vec3(uintBitsToFloat(CSG_TREE[index + 3]), uintBitsToFloat(CSG_TREE[index + 4]), uintBitsToFloat(CSG_TREE[index + 5]));

    return AABB(min, max);
}
CGSObject get_test_box(float time, vec3 pos) {
    PROFILE("get_test_box");
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
    PROFILE("get_test_sphere");
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 3.0)) * mat4_pos(pos));

    return CGSObject(
        mat,
        vec3(0.0),
        CGS_GEO_TYPE_SPHERE
    );
}

bool ray_hits_cgs_object(Ray ray, CGSObject object, out Interval intervall) {
    Ray model_space_ray = ray_to_model_space(ray, object.transform);

    if (object.type == CGS_GEO_TYPE_BOX) {
       return ray_aabb_intersect(model_space_ray, vec3(-0.5), vec3(0.5), intervall);
    } else if (object.type == CGS_GEO_TYPE_SPHERE) {
        return ray_sphere_intersect(model_space_ray, intervall);
    }

    return false;
}

bool pos_in_aabb(vec3 pos, vec3 min, vec3 max) {
    return min.x <= pos.x && pos.x <= max.x &&
    min.y <= pos.y && pos.y <= max.y &&
    min.z <= pos.z && pos.z <= max.z;
}

bool pos_in_sphere(vec3 pos, vec3 s_pos, float radius) {
    return distance(pos, s_pos) < radius;
}

bool exits_cgs_object(vec3 pos, CGSObject object) {
    PROFILE("exits_cgs_object");

    pos = (vec4(pos, 1.0) * object.transform).xyz;

    if (object.type == CGS_GEO_TYPE_BOX) {
        return pos_in_aabb(pos, vec3(-0.5), vec3(0.5));

    } else if (object.type == CGS_GEO_TYPE_SPHERE) {
        return length(pos) < 1.0;
    }

    return false;
}

bool cgs_bool_operation(bool exits_1, bool exits_2, uint operation) {
    if (operation == CGS_CHILD_TYPE_UNION) {
        return exits_1 || exits_2;
    }

    if (operation == CGS_CHILD_TYPE_REMOVE) {
        return exits_1 && !exits_2;
    }

    if (operation == CGS_CHILD_TYPE_INTERSECT) {
        return exits_1 && exits_2;
    }

    return false;
}

bool cgs_tree_at_pos(Ray ray, vec3 pos, in float current_t, in out float next_t) {
    PROFILE("cgs_tree_at_pos");
    int stack_len = 0;
    uint stack[MAX_CGS_TREE_DEPTH];
    uint operation_stack[MAX_CGS_TREE_DEPTH + 1];
    bool exits_1_stack[MAX_CGS_TREE_DEPTH + 1];
    operation_stack[0] = CGS_CHILD_TYPE_UNION;

    bool is_left = false;
    bool go_left = true;
    bool perform = false;

    uint current = 0;
    CGSChild child;
    bool exits_2 = false;
    bool exits = false;

    uint i = 0;

    while (i < MAX_CGS_RENDER_ITERATIONS) {
        i++;

        if (perform) {
            uint operation = operation_stack[stack_len];
            bool exits_1 = exits_1_stack[stack_len];

            exits = cgs_bool_operation(exits_1, exits_2, operation);

            if (stack_len <= 0) {
                if (is_left) {
                    is_left = false;
                } else {
                    break;
                }
            } else {
                stack_len--;
                current = stack[stack_len];

                if (is_left) {
                    exits_1_stack[stack_len] = exits;

                    perform = false;
                    go_left = false;
                } else {
                    exits_2 = exits;
                }
            }

            continue;
        }

        if (go_left) {
            child = get_csg_tree_child(current);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                AABB aabb = get_aabb(child.pointer);
                if (USE_AABB && !pos_in_aabb(pos, aabb.min, aabb.max)) {
                    exits_1_stack[stack_len] = false;;
                    go_left = false;
                }
                else {
                    stack[stack_len] = current;

                    stack_len++;
                    operation_stack[stack_len] = child.type;

                    current = child.pointer;

                    is_left = true;
                }

                Interval interval;
                if (ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                    if (interval.t_max > current_t) {
                        next_t = min(interval.t_min, next_t);
                    }
                }

            } else {
                AABB aabb = get_aabb(child.pointer);
                if (USE_AABB && !pos_in_aabb(pos, aabb.min, aabb.max)) {
                    exits_1_stack[stack_len] = false;
                } else {
                    CGSObject object = get_csg_tree_object(child.pointer);
                    exits_1_stack[stack_len] = exits_cgs_object(pos, object);
                }

                Interval interval;
                if (ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                    if (interval.t_max > current_t) {
                        next_t = min(interval.t_min, next_t);
                    }
                }

                go_left = false;
            }
        } else {
            child = get_csg_tree_child(current + 1);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                AABB aabb = get_aabb(child.pointer);
                if (USE_AABB && !pos_in_aabb(pos, aabb.min, aabb.max)) {
                    exits_2 = false;
                    perform = true;
                }
                else {
                    stack[stack_len] = current;
                    stack_len++;

                    current = child.pointer;
                    is_left = false;
                    go_left = true;
                }

                Interval interval;
                if (ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                    if (interval.t_max > current_t) {
                        next_t = min(interval.t_min, next_t);
                    }
                }

            } else {
                AABB aabb = get_aabb(child.pointer);
                if (USE_AABB && !pos_in_aabb(pos, aabb.min, aabb.max)) {
                    exits_2 = false;
                } else {
                    CGSObject object = get_csg_tree_object(child.pointer);
                    exits_2 = exits_cgs_object(pos, object);
                }

                Interval interval;
                if (ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                    if (interval.t_max > current_t) {
                        next_t = min(interval.t_min, next_t);
                    }
                }

                perform = true;
            }
        }
    }

    return exits;
}

#endif // _CGS_GLSL_