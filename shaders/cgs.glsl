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


#define CGS_CHILD_TYPE_NONE 0
#define CGS_CHILD_TYPE_UNION 1
#define CGS_CHILD_TYPE_REMOVE 2
#define CGS_CHILD_TYPE_INTERSECT 3
#define CGS_CHILD_TYPE_MAX_NODE 3
#define CGS_CHILD_TYPE_VOXEL 4
#define CGS_CHILD_TYPE_BOX 5
#define CGS_CHILD_TYPE_SPHERE 6

#define MAX_CGS_TREE_DEPTH 4
#define MAX_CGS_RENDER_ITERATIONS 10
#define MAX_INTERVAL_LIST 10

struct AABB {
    vec3 min;
    vec3 max;
};

struct CGSChild {
    uint pointer;
    uint type;
};

struct CGSObject {
    mat4 transform;
    vec3 data;
    uint material;
};

struct VoxelField {
    uint start;
};


AABB get_aabb(uint index) {
    PROFILE("get_aabb");

    vec3 min = vec3(uintBitsToFloat(CSG_TREE[index    ]), uintBitsToFloat(CSG_TREE[index + 1]), uintBitsToFloat(CSG_TREE[index + 2]));
    vec3 max = vec3(uintBitsToFloat(CSG_TREE[index + 3]), uintBitsToFloat(CSG_TREE[index + 4]), uintBitsToFloat(CSG_TREE[index + 5]));

    return AABB(min, max);
}

CGSChild get_csg_tree_child(uint index) {
    PROFILE("get_csg_tree_child");

    uint data = CSG_TREE[index + 6];
    uint pointer = data >> 3;      // 29 Bit
    uint type = data & uint(7);    //  3 Bit

    return CGSChild(pointer, type);
}

CGSObject get_csg_tree_object(uint index) {
    PROFILE("get_csg_tree_object");

    mat4 transform = mat4(
        uintBitsToFloat(CSG_TREE[index     + 6]), uintBitsToFloat(CSG_TREE[index + 4 + 6]), uintBitsToFloat(CSG_TREE[index +  8 + 6]), uintBitsToFloat(CSG_TREE[index +  12 + 6]),
        uintBitsToFloat(CSG_TREE[index + 1 + 6]), uintBitsToFloat(CSG_TREE[index + 5 + 6]), uintBitsToFloat(CSG_TREE[index +  9 + 6]), uintBitsToFloat(CSG_TREE[index +  13 + 6]),
        uintBitsToFloat(CSG_TREE[index + 2 + 6]), uintBitsToFloat(CSG_TREE[index + 6 + 6]), uintBitsToFloat(CSG_TREE[index + 10 + 6]), uintBitsToFloat(CSG_TREE[index +  14 + 6]),
        0.0, 0.0, 0.0, 1.0
    );
    vec3 data = vec3(uintBitsToFloat(CSG_TREE[index + 3 + 6]), uintBitsToFloat(CSG_TREE[index + 7 + 6]), uintBitsToFloat(CSG_TREE[index + 11 + 6]));
    uint material = CSG_TREE[index + 15 + 6];

    return CGSObject(transform, data, material);
}

VoxelField get_voxel_field(uint index) {
    PROFILE("get_voxel_field");

    uint start = CSG_TREE[index + 6];
    
    return VoxelField(start);
}

uint get_voxel_field_index(vec3 pos, AABB aabb) {
    uvec3 size = uvec3(round(aabb.max - aabb.min));
    uvec3 pos_in_aabb = uvec3(floor(pos - aabb.min));

    uint index = pos_in_aabb.x * size.y * size.z + pos_in_aabb.y * size.z + pos_in_aabb.z;
    return index;
}

uint get_voxel_value(uint start, uint index) {
    uint buffer_index = index >> 2;         // Upper bist (= index / 4)
    uint shift = (index & uint(3)) << 3;    // Lower 2 bits * 8 (= (index % 4) * 8;

    return (MATERIAL_BUFFER[start + buffer_index] >> shift) & 255;
}

CGSObject get_test_box(float time, vec3 pos) {
    PROFILE("get_test_box");
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 2.0;

    mat4 rot_mat = mat4_rotate_xyz(vec3(
        time_mod_rot(time, 0.2),
        time_mod_rot(time, 1.0),
        time_mod_rot(time, 0.4)));

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 1.5)) * rot_mat * mat4_pos(pos));

    return CGSObject(mat, vec3(1.0), 0);
}

CGSObject get_test_sphere(float time, vec3 pos) {
    PROFILE("get_test_sphere");
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 3.0)) * mat4_pos(pos));

    return CGSObject(
        mat,
        vec3(0.0),
        0
    );
}

bool ray_hits_cgs_object(Ray ray, CGSObject object, uint type, out Interval intervall) {
    Ray model_space_ray = ray_to_model_space(ray, object.transform);

    if (type == CGS_CHILD_TYPE_BOX) {
       return ray_aabb_intersect(model_space_ray, vec3(-0.5), vec3(0.5), intervall);
    } else if (type == CGS_CHILD_TYPE_SPHERE) {
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

bool exits_cgs_object(vec3 pos, CGSObject object, uint type) {
    PROFILE("exits_cgs_object");

    pos = (vec4(pos, 1.0) * object.transform).xyz;

    if (type == CGS_CHILD_TYPE_BOX) {
        return pos_in_aabb(pos, vec3(-0.5), vec3(0.5));

    } else if (type == CGS_CHILD_TYPE_SPHERE) {
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
    bool go_right = false;
    bool perform = false;

    uint current = 0;
    CGSChild child;
    AABB aabb;
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
                    go_right = true;
                } else {
                    exits_2 = exits;
                }
            }

            continue;
        }

        child = get_csg_tree_child(current + uint(go_right));
        aabb = get_aabb(child.pointer);

        if (USE_AABB && !pos_in_aabb(pos, aabb.min, aabb.max)) {
            if (!go_right) {
                exits_1_stack[stack_len] = false;;
                go_right = true;
            } else {
                exits_2 = false;
                perform = true;
            }
        } else {
            if (child.type <= CGS_CHILD_TYPE_MAX_NODE) {
                stack[stack_len] = current;
                stack_len++;
                current = child.pointer;

                if (!go_right) {
                    operation_stack[stack_len] = child.type;
                    is_left = true;
                } else {
                    is_left = false;
                    go_right = false;
                }

            } else if (child.type == CGS_CHILD_TYPE_VOXEL) {

                VoxelField voxle_filed = get_voxel_field(child.pointer);
                uint index = get_voxel_field_index(pos, aabb);
                uint voxel_value = get_voxel_value(voxle_filed.start, index);
                bool hit = voxel_value != uint(0);

                if (!go_right) {
                    exits_1_stack[stack_len] = hit;
                    go_right = true;
                } else {
                    exits_2 = hit;
                    perform = true;
                }
            } else {
                CGSObject object = get_csg_tree_object(child.pointer);
                bool hit = exits_cgs_object(pos, object, child.type);


                if (!go_right) {
                    exits_1_stack[stack_len] = hit;
                    go_right = true;
                } else {
                    exits_2 = hit;
                    perform = true;
                }
            }
        }
    }

    return exits;
}

struct IntervalList {
    Interval[MAX_INTERVAL_LIST] intervals;
    AABB[MAX_INTERVAL_LIST] aabbs;
    int len;
};

IntervalList init_interval_list() {
    Interval intervals[MAX_INTERVAL_LIST];
    AABB aabbs[MAX_INTERVAL_LIST];
    return IntervalList(intervals, aabbs, 0);
}

// From https://www.geeksforgeeks.org/search-insert-position-of-k-in-a-sorted-array/
int binary_search_interval_list(IntervalList list, float t)
{
    // Lower and upper bounds
    int start = 0;
    int end = list.len - 1;
    // Traverse the search space
    while (start <= end) {
        int mid = (start + end) / 2;

        if (list.intervals[mid].t_min < t) {
            start = mid + 1;
        } else {
            end = mid - 1;
        }
    }
    // Return insert position
    return end + 1;
}

IntervalList insert_into_list(IntervalList list, Interval interval, AABB aabb) {

    uint index = binary_search_interval_list(list, interval.t_min);

    for (uint i = list.len; i > index; i--) {
        list.intervals[i] = list.intervals[i - 1];
        list.aabbs[i] = list.aabbs[i - 1];
    }

    list.intervals[index] = interval;
    list.aabbs[index] = aabb;
    list.len += 1;

    return list;
}

void cgs_tree_interval_list(Ray ray, out IntervalList list) {
    PROFILE("cgs_tree_interval_list");

    list.len = 0;

    AABB aabb = get_aabb(0);
    Interval interval;
    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
        return;
    }

    uint stack[MAX_CGS_TREE_DEPTH];
    stack[0] = 0;
    stack[1] = 1;
    int stack_len = 2;

    CGSChild child;

    uint i = 0;
    while (stack_len > 0 && i < MAX_CGS_RENDER_ITERATIONS) {
        i++;

        stack_len -= 1;
        child = get_csg_tree_child(stack[stack_len]);
        aabb = get_aabb(child.pointer);

        if (ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {

            if (child.type > CGS_CHILD_TYPE_MAX_NODE) {
                list = insert_into_list(list, interval, aabb);
            } else {
                stack[stack_len] = child.pointer;
                stack[stack_len + 1] = child.pointer + 1;
                stack_len += 2;
            }
        }
    }
}

bool cgs_tree_next_interval(Ray ray, float current_t, out Interval interval, out AABB aabb) {
    PROFILE("cgs_tree_interval_list");

    aabb = get_aabb(0);
    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
        return false;
    }
    interval.t_min = FLOAT_POS_INF;

    uint stack[MAX_CGS_TREE_DEPTH];
    stack[0] = 0;
    stack[1] = 1;
    int stack_len = 2;

    CGSChild child;

    uint i = 0;
    bool hit = false;
    while (stack_len > 0 && i < MAX_CGS_RENDER_ITERATIONS) {
        i++;

        stack_len -= 1;
        child = get_csg_tree_child(stack[stack_len]);
        AABB new_aabb = get_aabb(child.pointer);

        Interval new_interval;
        if (ray_aabb_intersect(ray, new_aabb.min, new_aabb.max, new_interval)) {

            if (new_interval.t_max > current_t) {
                if (child.type > CGS_CHILD_TYPE_MAX_NODE) {
                    if (interval.t_min > new_interval.t_min) {
                        interval = new_interval;
                        aabb = new_aabb;

                        hit = true;
                    }
                } else {
                    stack[stack_len] = child.pointer;
                    stack[stack_len + 1] = child.pointer + 1;
                    stack_len += 2;
                }
            }
        }
    }

    return hit;
}

#endif // _CGS_GLSL_