#ifndef _RAY_GLSL_
#define _RAY_GLSL_

#define FLOAT_POS_INF uintBitsToFloat(0x7F800000)
#define FLOAT_NEG_INF uintBitsToFloat(0xFF800000)

struct Ray{
    vec3 pos;
    vec3 dir;
    vec3 odir; // = 1 / dir
};

struct Interval {
    float t_min;
    float t_max;
};

Ray init_ray(vec3 pos, vec3 dir, vec2 coord, vec2 res) {
    vec2 uv = ((coord * 2 - res) / res.y) * vec2(-1);

    vec3 ro = pos;
    vec3 fwd = dir;
    vec3 up = vec3(0.,0.,1.);
    vec3 right = normalize(cross(up,fwd));
    up = cross(fwd,right);
    vec3 rd = right * uv.x + up * uv.y + fwd;
    rd = normalize(rd);

    return Ray(ro, rd, vec3(1) / rd);
}

Interval init_interval() {
    return Interval(FLOAT_POS_INF, FLOAT_NEG_INF);
}

vec3 get_ray_pos(Ray ray, float t) {
    return ray.pos + ray.dir * t;
}

bool ray_aabb_intersect(in Ray ray, in vec3 min_pos, in vec3 max_pos, out Interval intervall) {
    PROFILE("ray_aabb_intersect");
    vec3 is_positive = vec3(ray.odir.x > 0, ray.odir.y > 0, ray.odir.z >= 0); // ray.odir = 1.0 / ray.dir
    vec3 is_negative = 1.0f - is_positive;

    vec3 left_side  = is_positive * min_pos + is_negative * max_pos;
    vec3 right_side = is_positive * max_pos + is_negative * min_pos;

    vec3 left_side_times_one_over_dir  = (left_side  - ray.pos) * ray.odir;
    vec3 right_side_times_one_over_dir = (right_side - ray.pos) * ray.odir;

    intervall.t_min = max(left_side_times_one_over_dir.x, max(left_side_times_one_over_dir.y, left_side_times_one_over_dir.z));
    intervall.t_max = min(right_side_times_one_over_dir.x, min(right_side_times_one_over_dir.y, right_side_times_one_over_dir.z));

    // vec3 directionSign = sign(odir);
    // sideMin = vec3(leftSideTimesOneOverDir.x == tMin, leftSideTimesOneOverDir.y == tMin, leftSideTimesOneOverDir.z == tMin) * directionSign;
    // sideMax = vec3(rightSideTimesOneOverDir.x == tMax, rightSideTimesOneOverDir.y == tMax, rightSideTimesOneOverDir.z == tMax) * directionSign;

    return intervall.t_max > 0 && intervall.t_max > intervall.t_min;
}

Ray ray_to_model_space(Ray ray, mat4 transform) {
    PROFILE("ray_to_model_space");
    vec3 new_pos = (vec4(ray.pos, 1.0) * transform).xyz;
    vec3 new_dir = (vec4(ray.dir, 0.0) * transform).xyz;

    return Ray(new_pos, new_dir, vec3(1) / new_dir);
}

// From
// https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection.html

bool solve_quadratic(float a, float b, float c, out float x0, out float x1) {
    float discr = b * b - 4 * a * c;
    if (discr < 0) {
        return false;
    } else if (discr == 0) {
        x0 = x1 = -0.5 * b / a;
    } else {
        float q = (b > 0) ?
        -0.5 * (b + sqrt(discr)):
        -0.5 * (b - sqrt(discr));
        x0 = q / a;
        x1 = c / q;
    }
    if (x0 > x1) {
        // Should use swap
        float temp = x0;
        x0 = x1;
        x1 = temp;
    };

    return true;
}

bool ray_sphere_intersect(Ray ray, out Interval intervall) {
    PROFILE("ray_sphere_intersect");
    #if 0
        // Geometric solution
        vec3 L = ray.pos;
        float tca = dot(L, ray.dir);
        // if (tca < 0) return false;
        float d2 = dot(L, L) - tca * tca;
        if (d2 > 1.0) return false;
        float thc = sqrt(1.0 - d2);
        t_min = tca - thc;
        t_max = tca + thc;
    #else
        // Analytic solution
        vec3 L = ray.pos;
        float a = dot(ray.dir, ray.dir);
        float b = 2 * dot(ray.dir, L);
        float c = dot(L, L) - 1.0;
        if (!solve_quadratic(a, b, c, intervall.t_min, intervall.t_max)) return false;
    #endif

    if (intervall.t_min > intervall.t_max) {
        // Should use swap
        float temp = intervall.t_min;
        intervall.t_min = intervall.t_max;
        intervall.t_max = temp;
    }

    return intervall.t_max > 0 && intervall.t_max > intervall.t_min;
}

#endif // _RAY_GLSL_