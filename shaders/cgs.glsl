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

#define CGS_GEO_TYPE_BOX 0
#define CGS_GEO_TYPE_SPHERE 1
#define CGS_GEO_TYPE_CAPSULE 2

#define CGS_CHILD_TYPE_GEO 0
#define CGS_CHILD_TYPE_UNION 1
#define CGS_CHILD_TYPE_REMOVE 2
#define CGS_CHILD_TYPE_INTERSECT 3

#define MAX_CGS_TREE_DEPTH 10
#define MAX_CGS_RENDER_ITERATIONS 10
#define MAX_CGS_INTERVALL_LIST 5

#define VOXEL_SIZE 10.0

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
    PROFILE("get_csg_tree_child");

    uint data = CSG_TREE[index];
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

AABB get_node_aabb(uint index) {
    return get_aabb(index + 2);
}

AABB get_leaf_aabb(uint index) {
    return get_aabb(index + 16);
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

Interval cgs_t_interval_operation(Interval left, Interval right, uint operation) {
    PROFILE("cgs_t_interval_operation");

    float t_min, t_max;
    if (operation == CGS_CHILD_TYPE_UNION) {
        t_min = min(left.t_min, right.t_min);
        t_max = max(left.t_max, right.t_max);

    } else if (operation == CGS_CHILD_TYPE_REMOVE) {
        if (right.t_min < left.t_min && left.t_min < right.t_max) {
            if (right.t_min < left.t_max && left.t_max < right.t_max) {
                t_min = FLOAT_POS_INF;
                t_max = FLOAT_NEG_INF;
            } else {
                t_min = right.t_max;
                t_max = left.t_max;
            }
        } else {
            if (right.t_min < left.t_max && left.t_max < right.t_max) {
                t_min = left.t_min;
                t_max = right.t_min;
            } else {
                t_min = left.t_min;
                t_max = left.t_max;
            }
        }

        if (right.t_min < left.t_max && left.t_max < right.t_max) {
            t_max = right.t_min;
        } else {
            t_max = left.t_max;
        }

    } else if (operation == CGS_CHILD_TYPE_INTERSECT) {
        t_min = max(left.t_min, right.t_min);
        t_max = min(left.t_max, right.t_max);
    }

    return Interval(t_min, t_max);
}


bool exits_cgs_object(vec3 pos, CGSObject object);
Interval ray_hits_cgs_tree(Ray ray, out uint i) {
    PROFILE("ray_hits_cgs_tree");

    Interval result = init_interval();

    AABB aabb = get_node_aabb(0);
    Interval interval;
    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
        return result;
    }

    int stack_len = 0;
    uint stack[MAX_CGS_TREE_DEPTH];
    uint operation_stack[MAX_CGS_TREE_DEPTH + 1];
    Interval left_stack[MAX_CGS_TREE_DEPTH + 1];
    Interval right = init_interval();

    operation_stack[0] = CGS_CHILD_TYPE_UNION;
    bool is_left = false;
    bool go_left = true;
    bool perform = false;

    uint current = 0;
    CGSChild child;

    i = 0;
    while (i < MAX_CGS_RENDER_ITERATIONS) {
        i++;

        if (perform) {

            uint operation = operation_stack[stack_len];
            Interval left = left_stack[stack_len];

            result = cgs_t_interval_operation(left, right, operation);

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
                    left_stack[stack_len] = result;

                    perform = false;
                    go_left = false;
                } else {
                    right = result;
                }
            }

        }
        else {
            if (go_left) {
                child = get_csg_tree_child(current);

                if (child.type != CGS_CHILD_TYPE_GEO) {

                    AABB aabb = get_node_aabb(child.pointer);
                    Interval interval;
                    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                        left_stack[stack_len] = init_interval();
                        go_left = false;
                    } else {
                        stack[stack_len] = current;

                        stack_len++;
                        operation_stack[stack_len] = child.type;

                        current = child.pointer;

                        is_left = true;
                    }

                } else {

                    AABB aabb = get_leaf_aabb(child.pointer);
                    Interval aabb_interval;
                    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, aabb_interval)) {
                        left_stack[stack_len] = init_interval();
                    }
                    else {
                        if (ONLY_RENDER_AABB) {
                            left_stack[stack_len] = aabb_interval;
                        } else {
                            CGSObject object = get_csg_tree_object(child.pointer);

                            Interval left_intervall;
                            bool hit = ray_hits_cgs_object(ray, object, left_intervall);
                            if (hit) {
                                if (ONLY_RENDER_CGS) {
                                    left_stack[stack_len] = left_intervall;
                                } else {
                                    vec3 start_pos = get_ray_pos(ray, aabb_interval.t_min) * VOXEL_SIZE;
                                    DDA dda = init_DDA(ray, start_pos, ivec3(100000));

                                    bool wait_in_side = true;
                                    for (uint j = 0; j < 1000; j++) {
                                        dda = step_DDA(dda);

                                        bool in_side = exits_cgs_object(floor(dda.pos) / VOXEL_SIZE, object);
                                        if (wait_in_side) {
                                            if (in_side) {
                                                float t = get_DDA_t(dda) / VOXEL_SIZE;
                                                left_intervall.t_min = aabb_interval.t_min + t;
                                                wait_in_side = false;
                                            }
                                        } else {
                                            if (!in_side) {
                                                float t = get_DDA_t(dda) / VOXEL_SIZE;
                                                left_intervall.t_max = aabb_interval.t_min + t;
                                                break;
                                            }
                                        }

                                    }

                                    left_stack[stack_len] = left_intervall;
                                }
                            } else {
                                left_stack[stack_len] = init_interval();
                            }
                        }
                    }

                    go_left = false;
                }
            } else {
                child = get_csg_tree_child(current + 1);

                if (child.type != CGS_CHILD_TYPE_GEO) {

                    AABB aabb = get_node_aabb(child.pointer);
                    Interval interval;
                    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                        right = init_interval();
                        perform = true;
                    }
                    else {
                        stack[stack_len] = current;
                        stack_len++;

                        current = child.pointer;
                        is_left = false;
                        go_left = true;
                    }

                } else {

                    AABB aabb = get_leaf_aabb(child.pointer);
                    Interval aabb_interval;
                    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, aabb_interval)) {
                        right = init_interval();
                    }
                    else {
                        if (ONLY_RENDER_AABB) {
                            right = aabb_interval;
                        } else {
                            CGSObject object = get_csg_tree_object(child.pointer);

                            Interval right_interval;
                            bool hit = ray_hits_cgs_object(ray, object, right_interval);
                            if (hit) {
                                if (ONLY_RENDER_CGS) {
                                    right = right_interval;
                                } else {
                                    vec3 start_pos = get_ray_pos(ray, aabb_interval.t_min) * VOXEL_SIZE;
                                    DDA dda = init_DDA(ray, start_pos, ivec3(100000));

                                    bool wait_in_side = true;
                                    for (uint j = 0; j < 1000; j++) {
                                        dda = step_DDA(dda);

                                        bool in_side = exits_cgs_object(floor(dda.pos) / VOXEL_SIZE, object);
                                        if (wait_in_side) {
                                            if (in_side) {
                                                float t = get_DDA_t(dda) / VOXEL_SIZE;
                                                right_interval.t_min = aabb_interval.t_min + t;
                                                wait_in_side = false;
                                            }
                                        } else {
                                            if (!in_side) {
                                                float t = get_DDA_t(dda) / VOXEL_SIZE;
                                                right_interval.t_max = aabb_interval.t_min + t;
                                                break;
                                            }
                                        }

                                    }

                                    left_stack[stack_len] = right_interval;
                                }
                            } else {
                                right = init_interval();
                            }
                        }
                    }

                    perform = true;
                }
            }
        }
    }

    return result;
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

bool cgs_tree_at_pos(vec3 pos) {
    PROFILE("cgs_tree_at_pos");

    int stack_len = 0;
    uint stack[MAX_CGS_TREE_DEPTH];
    uint operation_stack[MAX_CGS_TREE_DEPTH + 1];
    bool exits_1_stack[MAX_CGS_TREE_DEPTH + 1];
    operation_stack[0] = CGS_CHILD_TYPE_UNION;

    bool is_left = false;
    bool left = true;
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
                    left = false;
                } else {
                    exits_2 = exits;
                }
            }

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

                exits_1_stack[stack_len] = exits_cgs_object(pos, object);

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

                exits_2 = exits_cgs_object(pos, object);

                perform = true;
            }
        }
    }

    return exits;
}

#endif // _CGS_GLSL_