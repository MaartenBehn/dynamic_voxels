#ifndef _CGS_GLSL_
#define _CGS_GLSL_

#include "binding.glsl"
#include "ray.glsl"
#include "easing.glsl"
#include "mat_helper.glsl"
#include "debug.glsl"
#include "dda.glsl"
#include "aabb.glsl"
#include "interval.glsl"

#define ONLY_RENDER_AABB false
#define ONLY_RENDER_CGS false

#define CGS_CHILD_TYPE_NONE 0
#define CGS_CHILD_TYPE_UNION 1
#define CGS_CHILD_TYPE_REMOVE 2
#define CGS_CHILD_TYPE_INTERSECT 3
#define CGS_CHILD_TYPE_MAX_NODE 3
#define CGS_CHILD_TYPE_TRANSFORM 4
#define CGS_CHILD_TYPE_BOX 5
#define CGS_CHILD_TYPE_SPHERE 6
#define CGS_CHILD_TYPE_VOXEL_GIRD 7

#define CSG_DATA_AABB_SIZE 6
#define CSG_DATA_TRANSFORM_SIZE 12
#define CSG_DATA_VOEXL_GIRD_SIZE 6

#define MAX_CGS_TREE_DEPTH 4
#define MAX_CGS_RENDER_ITERATIONS 10

#define MATERIAL_NONE 0


struct CGSChild {
    uint pointer;
    uint type;
};

struct CGSTransform {
    mat4 transform;
};

struct CGSObject {
    mat4 transform;
    uint material;
};

struct VoxelGrid {
    AABB aabb;
    uvec3 size;
};

CGSChild get_csg_tree_child(uint index) {
    uint data = CSG_TREE[index];
    uint pointer = data >> 4; // 28 Bit
    uint type = data & uint(15); //  4 Bit

    return CGSChild(pointer, type);
}

AABB get_aabb(uint index) {
    vec3 min = vec3(uintBitsToFloat(CSG_TREE[index]), uintBitsToFloat(CSG_TREE[index + 1]), uintBitsToFloat(CSG_TREE[index + 2]));
    vec3 max = vec3(uintBitsToFloat(CSG_TREE[index + 3]), uintBitsToFloat(CSG_TREE[index + 4]), uintBitsToFloat(CSG_TREE[index + 5]));

    return AABB(min, max);
}

mat4 get_mat4_form_csg_tree(uint index) {
    return mat4(
        uintBitsToFloat(CSG_TREE[index]), uintBitsToFloat(CSG_TREE[index + 3]), uintBitsToFloat(CSG_TREE[index + 6]), uintBitsToFloat(CSG_TREE[index + 9]),
        uintBitsToFloat(CSG_TREE[index + 1]), uintBitsToFloat(CSG_TREE[index + 4]), uintBitsToFloat(CSG_TREE[index + 7]), uintBitsToFloat(CSG_TREE[index + 10]),
        uintBitsToFloat(CSG_TREE[index + 2]), uintBitsToFloat(CSG_TREE[index + 5]), uintBitsToFloat(CSG_TREE[index + 8]), uintBitsToFloat(CSG_TREE[index + 11]),
        0.0, 0.0, 0.0, 1.0
    );
}

CGSTransform get_csg_tree_transform(uint index) {
    mat4 transform = get_mat4_form_csg_tree(index + CSG_DATA_AABB_SIZE);

    return CGSTransform(transform);
}

CGSObject get_csg_tree_object(uint index) {
    mat4 transform = get_mat4_form_csg_tree(index + CSG_DATA_AABB_SIZE);
    uint material = CSG_TREE[index + CSG_DATA_AABB_SIZE + CSG_DATA_TRANSFORM_SIZE];

    return CGSObject(transform, material);
}

uint get_voxel_value(uint start, uint index) {
    uint buffer_index = index >> 2; // Upper bist (= index / 4)
    uint shift = (index & uint(3)) << 3; // Lower 2 bits * 8 (= (index % 4) * 8;

    return (CSG_TREE[start + buffer_index] >> shift) & 255;
}

VoxelGrid get_voxel_grid(uint index) {
    AABB aabb = get_aabb(index + CSG_DATA_AABB_SIZE);
    uvec3 size = uvec3(round(aabb.max - aabb.min));
    
    return VoxelGrid(aabb, size);
}

bool in_voxel_grid_bounds(VoxelGrid grid, uvec3 pos) {
    return pos_in_aabb(vec3(pos), grid.aabb.min, grid.aabb.max); 
}

uint get_voxel_grid_value(VoxelGrid grid, uvec3 pos, uint start) { 
    uint index = pos.x * grid.size.y * grid.size.z + pos.y * grid.size.z + pos.z;
    return get_voxel_value(start + CSG_DATA_AABB_SIZE + CSG_DATA_VOEXL_GIRD_SIZE, index);
}

CGSObject get_test_box(float time, vec3 pos) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 2.0;

    mat4 rot_mat = mat4_rotate_xyz(vec3(
                time_mod_rot(time, 0.2),
                time_mod_rot(time, 1.0),
                time_mod_rot(time, 0.4)));

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 1.5)) * rot_mat * mat4_pos(pos));

    return CGSObject(mat, 1);
}

CGSObject get_test_sphere(float time, vec3 pos) {
    float scale = 1.0 + ease_cubic_in_out(ease_loop(time_mod(time, 1.0))) * 0.1;

    mat4 mat = inverse(mat4_scale(vec3(scale, 2.0, 3.0)) * mat4_pos(pos));

    return CGSObject(mat, 1);
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

bool pos_in_sphere(vec3 pos, vec3 s_pos, float radius) {
    return distance(pos, s_pos) < radius;
}

bool exits_cgs_object(vec3 pos, CGSObject object, uint type) {
    pos = (vec4(pos, 1.0) * object.transform).xyz;

    if (type == CGS_CHILD_TYPE_BOX) {
        return pos_in_aabb(pos, vec3(-0.5), vec3(0.5));
    } else if (type == CGS_CHILD_TYPE_SPHERE) {
        return dot(pos, pos) < 1.0;
    }

    return false;
}

uint cgs_material_operation(uint material_1, uint material_2, uint operation) {
    if (operation == CGS_CHILD_TYPE_UNION) {
        if (material_1 != 0) {
            return material_1;
        }

        if (material_2 != 0) {
            return material_2;
        }
    }

    if (operation == CGS_CHILD_TYPE_REMOVE) {
        if (material_1 != 0 && material_2 == 0) {
            return material_1;
        }
    }

    if (operation == CGS_CHILD_TYPE_INTERSECT) {
        if (material_1 != 0 && material_2 != 0) {
            return material_1;
        }
    }

    return 0;
}
#endif // _CGS_GLSL_

