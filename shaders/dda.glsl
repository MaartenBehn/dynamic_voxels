#ifndef _DDA_GLSL_
#define _DDA_GLSL_

#include "ray.glsl"

struct DDA {
    vec3 pos;
    vec3 delta_dist;
    vec3 delta;
    vec3 step;
    vec3 side_dist;
    vec3 mask;
    vec3 lower_bound;
    vec3 upper_bound;
    bool out_of_bounds;
};

DDA init_DDA(in Ray ray, in vec3 start_pos, in vec3 lower_bound, in vec3 upper_bound) {
    vec3 cell = floor(start_pos);
    vec3 delta_dist = abs(vec3(length(ray.dir)) / ray.dir);
    vec3 delta = abs(1.0 / ray.dir);
    vec3 step = sign(ray.dir);
    vec3 side_dist = (step * (cell - start_pos) + (step * 0.5) + 0.5) * delta_dist;
    vec3 mask;

    return DDA(start_pos, delta_dist, delta, step, side_dist, mask, lower_bound, upper_bound, false);
}

DDA step_DDA(in DDA dda, in uint steps) {
    PROFILE("step_DDA");

    if (steps > 1)
    {
        if (steps > 2)
        {
            if (steps > 3)
            {
                dda.mask = floor( float(steps - 2) / dda.delta );
                dda.side_dist += dda.mask * dda.delta_dist;
                dda.pos += dda.mask * dda.step;
            }

            dda.mask = vec3(lessThanEqual(dda.side_dist.xyz, min(dda.side_dist.yzx, dda.side_dist.zxy)));
            dda.side_dist += dda.mask * dda.delta_dist;
            dda.pos += dda.mask * dda.step;
        }

        dda.mask = vec3(lessThanEqual(dda.side_dist.xyz, min(dda.side_dist.yzx, dda.side_dist.zxy)));
        dda.side_dist += dda.mask * dda.delta_dist;
        dda.pos += dda.mask * dda.step;
    }

    dda.mask = vec3(lessThanEqual(dda.side_dist.xyz, min(dda.side_dist.yzx, dda.side_dist.zxy)));
    dda.side_dist += dda.mask * dda.delta_dist;
    dda.pos += dda.mask * dda.step;

    dda.out_of_bounds =
      (dda.mask.x != 0 && (dda.pos.x < dda.lower_bound.x || dda.pos.x > dda.upper_bound.x)
    || dda.mask.y != 0 && (dda.pos.y < dda.lower_bound.y || dda.pos.y > dda.upper_bound.y)
    || dda.mask.z != 0 && (dda.pos.z < dda.lower_bound.z || dda.pos.z > dda.upper_bound.z));

    return dda;
}

float get_DDA_t(in DDA dda) {
    vec3 side_dist = dda.mask * dda.side_dist;
    return side_dist.x + side_dist.y + side_dist.z;
}

#endif // __DDA_GLSL__