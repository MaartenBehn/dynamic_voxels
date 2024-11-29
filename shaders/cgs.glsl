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
#define MAX_CGS_INTERVALL_LIST 5

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

struct IntervalList {
    Interval interval[MAX_CGS_INTERVALL_LIST];
    uint len;
};

IntervalList init_interval_list() {
    Interval interval[MAX_CGS_INTERVALL_LIST];

    return IntervalList(interval, 0);
}

IntervalList init_interval_list_with_value(Interval val) {
    Interval interval[MAX_CGS_INTERVALL_LIST];
    interval[0] = val;

    return IntervalList(interval, 1);
}

void push_interval_list(Interval val, in out IntervalList list) {
    list.interval[list.len] = val;
    list.len++;
}

Interval pop_interval_list(in out IntervalList list) {
    list.len--;
    return list.interval[list.len];
}

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

IntervalList cgs_union_interval(IntervalList left, IntervalList right) {
    PROFILE("cgs_union_interval");

    if (left.len == 0){
        return right;
    }

    if (right.len == 0){
        return left;
    }

    uint index_left = 0;
    uint index_right = 0;
    Interval current;
    IntervalList result = init_interval_list();

    bool left_t_smaller = left.interval[index_left].t_min < right.interval[index_right].t_min;
    if (left_t_smaller) {
        current = left.interval[index_left];
        index_left++;

    } else {
        current = right.interval[index_right];
        index_right++;
    }

    for(uint i = 0; i < MAX_CGS_INTERVALL_LIST * 2; i++) {
        Interval other;

        bool left_in_bound = index_left < left.len;
        bool right_in_bound = index_right < right.len;
        left_t_smaller = left.interval[index_left].t_min < right.interval[index_right].t_min;

        if ((left_t_smaller && left_in_bound) || !right_in_bound) {
            if (!left_in_bound) {
                push_interval_list(current, result);
                break;
            }

            other = left.interval[index_left];
            index_left++;
        } else {
            other = right.interval[index_right];
            index_right++;
        }

        bool intersects = current.t_min < other.t_max && current.t_max > other.t_min;
        if (intersects) {
            current.t_max = other.t_max;
        } else {
            push_interval_list(current, result);
            current = other;
        }
    }

    return result;
}

IntervalList cgs_remove_interval(IntervalList left, IntervalList right) {
    PROFILE("cgs_remove_interval");

    if (left.len == 0){
        return init_interval_list();
    } else if (right.len == 0) {
        return left;
    }

    Interval current = left.interval[0];
    Interval other = right.interval[0];
    uint next_left = 1;
    uint next_right = 1;

    IntervalList result = init_interval_list();

    for(uint i = 0; i < MAX_CGS_INTERVALL_LIST * 2; i++)  {

        // Cases
        // ----       | ----   |   ---- | ------- |   ----    |      ---- |
        //       ---- |   ---- | ----   |   ---   | --------- | ----      |
        // 0            1        2        3         4           5

        bool keep_front = current.t_min < other.t_min;
        bool keep_back = current.t_max > other.t_max;
        bool left_first = current.t_max < other.t_min;
        bool right_first = other.t_max < current.t_min;
        bool intersection = !left_first && !right_first;

        uint c;
        if (!intersection && left_first) {
            c = 0;
        } else if (intersection && keep_front && !keep_back) {
            c = 1;
        } else if (intersection && !keep_front && keep_back) {
            c = 2;
        } else if (intersection && keep_front && keep_back) {
            c = 3;
        } else if (intersection && !keep_front && !keep_back) {
            c = 4;
        } else if (!intersection && right_first) {
            c = 5;
        }

        // Create front bit and push it
        if (c == 1 || c == 3) {
            Interval sub = current;
            sub.t_max = other.t_min;

            push_interval_list(sub, result);
        }

        // Make current end bit
        if (c == 2 || c == 3) {
            current.t_min = other.t_max;
        }

        // Push current
        if (c == 0) {
            push_interval_list(current, result);
        }

        bool left_out_of_bounds = next_left >= left.len;
        bool right_out_of_bounds = next_right >= right.len;
        bool both_out_of_bounds = left_out_of_bounds && right_out_of_bounds;

        // current = next left
        if ((c == 0 || c == 1 || c == 4) && !left_out_of_bounds) {
            current = left.interval[next_left];
            next_left++;
        }

        // other = next right
        if ((c == 2 || c == 3 || c == 5) && !right_out_of_bounds) {
            other = right.interval[next_right];
            next_right++;
        }


        if (both_out_of_bounds) {
            if (c == 0 || c == 1 || c == 4) {
                break;
            }

            if (c == 2 || c == 3 || c == 5) {
                push_interval_list(current, result);
                break;
            }
        }

        if (left_out_of_bounds && (c == 0 || c == 1 || c == 4)) {
            break;
        }

        if (right_out_of_bounds && (c == 2 || c == 3 || c == 5)) {
            push_interval_list(current, result);

            while (next_left < left.len) {
                push_interval_list(left.interval[next_left], result);
                next_left++;
            }

            break;
        }
    }

    return result;
}

IntervalList cgs_intersect_interval(IntervalList left, IntervalList right) {
    PROFILE("cgs_intersect_interval");

    IntervalList result = init_interval_list();

    if (left.len == 0 || right.len == 0){
        return result;
    }

    Interval current_left = left.interval[0];
    Interval current_right = right.interval[0];

    uint index_left = 1;
    uint index_right = 1;

    for(uint i = 0; i < MAX_CGS_INTERVALL_LIST * 2; i++)  {
        if (current_left.t_min < current_right.t_max && current_left.t_max > current_right.t_min) {

            Interval intersection;
            if (current_left.t_min > current_right.t_min) {
                intersection = current_left;
            } else {
                intersection = current_right;
            }

            if (current_left.t_max < current_right.t_max) {
                intersection.t_max = current_left.t_max;
            } else {
                intersection.t_max = current_right.t_max;
            }

            push_interval_list(intersection, result);
        }

        bool left_in_bound = index_left < left.len;
        bool right_in_bound = index_right < right.len;
        bool left_bt_smaller = left.interval[index_left].t_max < right.interval[index_right].t_max;

        if ((left_bt_smaller && left_in_bound) || !right_in_bound) {
            if (!left_in_bound) {
                break;
            }

            current_left = left.interval[index_left];
            index_left++;
        } else {
            current_right = right.interval[index_right];
            index_right++;
        }
    }

    return result;
}

IntervalList cgs_t_interval_operation(IntervalList left, IntervalList right, uint operation) {
    PROFILE("cgs_t_interval_operation");

    if (operation == CGS_CHILD_TYPE_UNION) {
        return cgs_union_interval(left, right);
    }

    if (operation == CGS_CHILD_TYPE_REMOVE) {
        return cgs_remove_interval(left, right);
    }

    if (operation == CGS_CHILD_TYPE_INTERSECT) {
        return cgs_intersect_interval(left, right);
    }

    return init_interval_list();
}


IntervalList ray_hits_cgs_tree(Ray ray) {
    PROFILE("ray_hits_cgs_tree");

    int stack_len = 0;
    uint stack[MAX_CGS_TREE_DEPTH];
    uint operation_stack[MAX_CGS_TREE_DEPTH + 1];
    IntervalList left_stack[MAX_CGS_TREE_DEPTH + 1];
    IntervalList right = init_interval_list();
    IntervalList result = init_interval_list();

    operation_stack[0] = CGS_CHILD_TYPE_UNION;
    bool is_left = false;
    bool go_left = true;
    bool perform = false;

    uint current = 0;
    CGSChild child;

    uint i = 0;
    while (i < MAX_CGS_RENDER_ITERATIONS) {
        i++;

        if (perform) {
            uint operation = operation_stack[stack_len];
            IntervalList left = left_stack[stack_len];

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

            continue;
        }

        if (go_left) {
            child = get_csg_tree_child(current);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                stack[stack_len] = current;

                stack_len++;
                operation_stack[stack_len] = child.type;

                current = child.pointer;

                is_left = true;
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);

                Interval left_intervall;
                bool hit = ray_hits_cgs_object(ray, object, left_intervall);
                if (hit) {
                    left_stack[stack_len] = init_interval_list_with_value(left_intervall);
                } else {
                    left_stack[stack_len] = init_interval_list();
                }

                // result = init_interval_list_with_value(left_intervall);
                // break;

                go_left = false;
            }
        } else {
            child = get_csg_tree_child(current + 1);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                stack[stack_len] = current;
                stack_len++;

                current = child.pointer;
                is_left = false;
                go_left = true;
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);

                Interval right_intervall;
                bool hit = ray_hits_cgs_object(ray, object, right_intervall);
                if (hit) {
                    right = init_interval_list_with_value(right_intervall);
                } else {
                    right = init_interval_list();
                }

                // result = init_interval_list_with_value(right_intervall);
                // break;

                perform = true;
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

bool exits_cgs_object(vec3 pos, CGSObject object, float padding) {
    PROFILE("exits_cgs_object");

    pos = (vec4(pos, 1.0) * object.transform).xyz;
    vec3 padding_vec = (object.transform * vec4(padding, padding, padding, 1.0)).xyz;

    if (object.type == CGS_GEO_TYPE_BOX) {
        return pos_in_aabb(pos, vec3(-0.5) - padding_vec, vec3(0.5) + padding_vec);

    } else if (object.type == CGS_GEO_TYPE_SPHERE) {
        return length(pos) < 1.0 + padding;
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

bool cgs_operation_padding(uint operation, out float padding_1, out float padding_2) {
    if (operation == CGS_CHILD_TYPE_UNION) {
        padding_1 = -0.2;
        padding_2 = -0.2;
    }

    if (operation == CGS_CHILD_TYPE_REMOVE) {
        padding_1 = -0.2;
        padding_2 = 0.2;
    }

    if (operation == CGS_CHILD_TYPE_INTERSECT) {
        padding_1 = -0.2;
        padding_2 = -0.2;
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


    float padding_1;
    float padding_2;
    cgs_operation_padding(operation_stack[0], padding_1, padding_2);

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

            cgs_operation_padding(operation_stack[stack_len], padding_1, padding_2);

            continue;
        }

        if (left) {
            child = get_csg_tree_child(current);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                stack[stack_len] = current;

                stack_len++;
                operation_stack[stack_len] = child.type;

                cgs_operation_padding(child.type, padding_1, padding_2);

                current = child.pointer;

                is_left = true;
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);

                exits_1_stack[stack_len] = exits_cgs_object(pos, object, padding_1);

                left = false;
            }
        } else {
            child = get_csg_tree_child(current + 1);

            if (child.type != CGS_CHILD_TYPE_GEO) {
                stack[stack_len] = current;
                stack_len++;

                cgs_operation_padding(child.type, padding_1, padding_2);

                current = child.pointer;
                is_left = false;
                left = true;
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);

                exits_2 = exits_cgs_object(pos, object, padding_2);

                perform = true;
            }
        }
    }

    return exits;
}


#endif // _CGS_GLSL_