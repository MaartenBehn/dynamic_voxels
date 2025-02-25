#ifndef _DDA_GLSL_
#define _DDA_GLSL_

#include "ray.glsl"
#include "aabb.glsl"

struct DDA {
    vec3 cell;
    vec3 delta_dist;
    vec3 step;
    vec3 side_dist;
    vec3 mask;
    vec3 lower_bound;
    vec3 upper_bound;
    bool out_of_bounds;
};

vec3 get_mask(vec3 side_dist) {
    return vec3(lessThanEqual(side_dist.xyz, min(side_dist.yzx, side_dist.zxy)));
}

DDA init_DDA(in Ray ray, in vec3 start_pos, in vec3 lower_bound, in vec3 upper_bound, in float scale) {
    //PROFILE("init_DDA");  

    vec3 cell = floor(start_pos / scale) * scale;
    vec3 delta_dist = abs(ray.odir);
    vec3 step = sign(ray.dir);
    vec3 side_dist = (step * (cell - start_pos) + (step * 0.5) + 0.5) * delta_dist;
    vec3 mask = get_mask(side_dist);

    return DDA(cell, delta_dist * scale, step * scale, side_dist, mask, lower_bound, upper_bound, false);
}

DDA step_DDA(in DDA dda) {
    //PROFILE("step_DDA");  

    dda.mask = get_mask(dda.side_dist);
    dda.side_dist += dda.mask * dda.delta_dist;
    dda.cell += dda.mask * dda.step;

    dda.out_of_bounds =
        (dda.mask.x != 0 && (dda.cell.x < dda.lower_bound.x || dda.cell.x > dda.upper_bound.x)
            || dda.mask.y != 0 && (dda.cell.y < dda.lower_bound.y || dda.cell.y > dda.upper_bound.y)
            || dda.mask.z != 0 && (dda.cell.z < dda.lower_bound.z || dda.cell.z > dda.upper_bound.z));

    return dda;
}

float get_DDA_t(in DDA dda) {
    vec3 side_dist = dda.mask * dda.side_dist;
    return side_dist.x + side_dist.y + side_dist.z;
}

struct DDA_INC {
    vec3 cell;
    vec3 delta_dist;
    vec3 step;
    vec3 side_dist;
    vec3 mask;
    vec3 lower_bound;
    vec3 upper_bound;
    bool out_of_bounds;
    float scale;
};

DDA_INC init_DDA_INC(in Ray ray, in vec3 start_pos, in vec3 lower_bound, in vec3 upper_bound, in float scale) {
    vec3 cell = start_pos;
    vec3 delta_dist = abs(ray.odir);
    vec3 step = sign(ray.dir);
    vec3 side_dist = (step * (cell - start_pos) + (step * 0.5) + 0.5) * delta_dist;
    vec3 mask = get_mask(side_dist);

    return DDA_INC(cell, delta_dist, step, side_dist, mask, lower_bound, upper_bound, false, scale);
}

DDA_INC step_DDA(in DDA_INC dda) {
    float new_scale = dda.scale * dda.scale;
    if ((dda.cell / new_scale) == vec3(0)) {
        dda.scale = new_scale;
    }

    dda.mask = get_mask(dda.side_dist);
    dda.side_dist += dda.mask * dda.delta_dist * dda.scale;
    dda.cell += dda.mask * dda.step * dda.scale;

    dda.out_of_bounds =
        (dda.mask.x != 0 && (dda.cell.x < dda.lower_bound.x || dda.cell.x > dda.upper_bound.x)
            || dda.mask.y != 0 && (dda.cell.y < dda.lower_bound.y || dda.cell.y > dda.upper_bound.y)
            || dda.mask.z != 0 && (dda.cell.z < dda.lower_bound.z || dda.cell.z > dda.upper_bound.z));

    return dda;
}

float get_DDA_INC_t(in DDA_INC dda) {
    vec3 side_dist = dda.mask * dda.side_dist;
    return side_dist.x + side_dist.y + side_dist.z;
}

float get_DDA_scale_from_AABB_dist(Ray ray, AABB aabb, float t_min) { 
    if (t_min < 0) {
        return 1;
    }

    vec3 aabb_closest_point = min(abs(ray.pos - aabb.min), abs(ray.pos - aabb.max));
    float dist_to_aabb = length(aabb_closest_point);
    return clamp(exp2(floor(dist_to_aabb / 50.0)), 1, 16);
}

#endif // __DDA_GLSL__

