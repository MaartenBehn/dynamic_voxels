#version 450 core

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

#include "binding.glsl"
#include "ray.glsl"
#include "cgs.glsl"
#include "dda.glsl"

#define RENDER_TEST_OBJECT false
#define RENDER_CSG_FULL_DDA true

#define SHOW_DDA_STEPS false
#define SHOW_DISTANCE false
#define SHOW_CSG_STEPS false

#define USE_AABB true

#define MAX_DDA_STEPS 500
#define MAX_DEPTH 300
#define EPSILON 0.0001

#define MAX_CGS_TREE_DEPTH 20
#define MAX_CGS_RENDER_ITERATIONS 40

#define TO_1D(pos, size) ((pos.z * size * size) + (pos.y * size) + pos.x)

void main () {
    //PROFILE("main");

    Ray ray = init_ray(POS, DIR, gl_GlobalInvocationID.xy, RES);

    vec4 color = vec4(ray.dir, 1);

    float time = TIME;

    if (RENDER_TEST_OBJECT) {
        CGSObject obj = get_test_sphere(time, vec3(0,0,0));
        Interval interval;
        if (ray_hits_cgs_object(ray, obj, CGS_CHILD_TYPE_SPHERE, interval)) {
            float t_min = interval.t_min;
            color = vec4(get_debug_color_gradient_from_float(t_min / 10.0), 1.0);
        }

        imageStore(img, ivec2(gl_GlobalInvocationID.xy), color);
        return;
    }
      
    if(RENDER_CSG_FULL_DDA) {
 
        uint stack[MAX_CGS_TREE_DEPTH];
        stack[0] = 0;
        Ray ray_stack[MAX_CGS_TREE_DEPTH];
        ray_stack[0] = ray;
        int stack_len = 1;

        float best_distance = FLOAT_POS_INF;
        vec4 new_color;
        float new_distance = FLOAT_POS_INF;
        uint larges_dda_step_counter = 0;
        uint smalest_dda_step_counter = 0;

        uint csg_step_counter = 0;
        uint dda_step_counter = 0;
        while (stack_len > 0 && csg_step_counter < MAX_CGS_RENDER_ITERATIONS) {
            csg_step_counter++;
            
            stack_len -= 1;
            ray = ray_stack[stack_len];
            CGSChild child = get_csg_tree_child(stack[stack_len]);
             
            if (child.type == CGS_CHILD_TYPE_UNION) {
                if (USE_AABB) {
                    AABB aabb = get_aabb(child.pointer);
                    Interval interval;
                    if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                        continue;
                    }
                }

                stack[stack_len] = child.pointer + CSG_DATA_AABB_SIZE;
                stack[stack_len + 1] = child.pointer + CSG_DATA_AABB_SIZE + 1;
                ray_stack[stack_len] = ray;
                ray_stack[stack_len + 1] = ray;
                stack_len += 2;

            } else if (child.type <= CGS_CHILD_TYPE_MAX_NODE) {


            } else if (child.type == CGS_CHILD_TYPE_TRANSFORM) {
                CGSTransform object = get_csg_tree_transform(child.pointer);
                ray = ray_to_model_space(ray, object.transform);
                
                stack[stack_len] = child.pointer + CSG_DATA_AABB_SIZE + CSG_DATA_TRANSFORM_SIZE;
                ray_stack[stack_len] = ray;
                stack_len += 1;

            } else if (child.type == CGS_CHILD_TYPE_VOXEL_GIRD) {
                VoxelGrid voxel_grid = get_voxel_grid(child.pointer);

                Interval interval;
                if (!ray_aabb_intersect(ray, voxel_grid.aabb.min, voxel_grid.aabb.max, interval)) {
                    continue;
                }

                float t_start = max(interval.t_min, 0) + EPSILON;

                vec3 start_pos = get_ray_pos(ray, t_start); 
                DDA dda = init_DDA(ray, start_pos, voxel_grid.aabb.min, voxel_grid.aabb.max, 1);

                dda_step_counter = 0;
                while (dda_step_counter < MAX_DDA_STEPS) { 
                    uint material = get_voxel_grid_value(voxel_grid, uvec3(dda.cell - voxel_grid.aabb.min), child.pointer);

                    if (material != 0) {
                        new_distance = get_DDA_t(dda) + t_start;
                        new_color = COLOR_BUFFER[material];
                        break;
                    }

                    dda = step_DDA(dda);

                    if (dda.out_of_bounds) {
                        break;
                    }

                    dda_step_counter++;
                }

            } else {
                CGSObject object = get_csg_tree_object(child.pointer);
                
                AABB aabb = get_aabb(child.pointer);
                Interval interval;
                if (!ray_aabb_intersect(ray, aabb.min, aabb.max, interval)) {
                    continue;
                }

                float t_start = max(interval.t_min, 0) + EPSILON;
                vec3 start_pos = get_ray_pos(ray, t_start); 
                DDA dda = init_DDA(ray, start_pos, aabb.min, aabb.max, 1);
                
                dda_step_counter = 0;
                while (dda_step_counter < MAX_DDA_STEPS) { 
                    if (exits_cgs_object(uvec3(dda.cell), object, child.type)) {
                        new_distance = get_DDA_t(dda) + t_start;
                        new_color = COLOR_BUFFER[object.material];
                        break;
                    }

                    dda = step_DDA(dda);

                    if (dda.out_of_bounds) {
                        break;
                    }

                    dda_step_counter++;
                }
            }

            if (best_distance > new_distance) {
                color = new_color;
                best_distance = new_distance;
            }
            larges_dda_step_counter = max(larges_dda_step_counter, dda_step_counter);
            smalest_dda_step_counter = min(smalest_dda_step_counter, dda_step_counter);
        }

        
        if (SHOW_DISTANCE && best_distance != FLOAT_POS_INF) {
            color = vec4(get_debug_color_gradient_from_float(best_distance / MAX_DEPTH), 1.0);
        } else  if (SHOW_DDA_STEPS) {
            color = vec4(get_debug_color_gradient_from_float(float(smalest_dda_step_counter) / MAX_DDA_STEPS), 1.0);
        } else  if (SHOW_CSG_STEPS) {
            color = vec4(get_debug_color_gradient_from_float(float(csg_step_counter) / MAX_CGS_RENDER_ITERATIONS), 1.0);
        }

        imageStore(img, ivec2(gl_GlobalInvocationID.xy), color);
        return;
    }
}



