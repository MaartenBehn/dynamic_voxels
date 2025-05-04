
struct Ray {
    float3 pos;
    float3 dir;
    float3 odir; // = 1 / dir
};

Ray init_ray(float3 pos, float3 dir, float2 res, float2 coord) {
    float2 uv = ((coord * 2 - res) / res.y) * float2(-1);

    float3 ro = pos;
    float3 fwd = dir;
    float3 up = float3(0., 0., 1.);
    float3 right = normalize(cross(up, fwd));
    up = cross(fwd, right);
    float3 rd = right * uv.x + up * uv.y + fwd;
    rd = normalize(rd);

    return Ray(ro, rd, float3(1) / rd);
}

float3 get_ray_pos(Ray ray, float t) {
    return ray.pos + ray.dir * t;
}

struct Interval {
    float t_min;
    float t_max;
};

#define FLOAT_POS_INF asfloat(0x7F800000)
#define FLOAT_NEG_INF asfloat(0xFF800000)

Interval init_interval() {
    return Interval(FLOAT_POS_INF, FLOAT_NEG_INF);
}

bool ray_aabb_intersect(in Ray ray, in float3 min_pos, in float3 max_pos, out Interval intervall) {
    float3 is_positive = float3(float(ray.odir.x > 0), float(ray.odir.y > 0), float(ray.odir.z >= 0)); // ray.odir = 1.0 / ray.dir
    float3 is_negative = 1.0f - is_positive;

    float3 left_side = is_positive * min_pos + is_negative * max_pos;
    float3 right_side = is_positive * max_pos + is_negative * min_pos;

    float3 left_side_times_one_over_dir = (left_side - ray.pos) * ray.odir;
    float3 right_side_times_one_over_dir = (right_side - ray.pos) * ray.odir;

    intervall.t_min = max(left_side_times_one_over_dir.x, max(left_side_times_one_over_dir.y, left_side_times_one_over_dir.z));
    intervall.t_max = min(right_side_times_one_over_dir.x, min(right_side_times_one_over_dir.y, right_side_times_one_over_dir.z));

    // vec3 directionSign = sign(odir);
    // sideMin = vec3(leftSideTimesOneOverDir.x == tMin, leftSideTimesOneOverDir.y == tMin, leftSideTimesOneOverDir.z == tMin) * directionSign;
    // sideMax = vec3(rightSideTimesOneOverDir.x == tMax, rightSideTimesOneOverDir.y == tMax, rightSideTimesOneOverDir.z == tMax) * directionSign;

    return intervall.t_max > 0 && intervall.t_max > intervall.t_min;
}
