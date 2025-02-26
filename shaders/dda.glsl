#ifndef _DDA_GLSL_
#define _DDA_GLSL_

#include "ray.glsl"

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
#endif // __DDA_GLSL__

