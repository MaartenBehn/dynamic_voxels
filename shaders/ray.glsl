#ifndef _RAY_GLSL_
#define _RAY_GLSL_

struct Ray{
    vec3 pos;
    vec3 dir;
    vec3 odir; // = 1 / dir
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

bool aabb_ray_test(in Ray ray, in vec3 min_pos, in vec3 max_pos, out float t_min, out float t_max) {
    vec3 is_positive = vec3(ray.odir.x > 0, ray.odir.y > 0, ray.odir.z >= 0); // ray.odir = 1.0 / ray.dir
    vec3 is_negative = 1.0f - is_positive;

    vec3 left_side  = is_positive * min_pos + is_negative * max_pos;
    vec3 right_side = is_positive * max_pos + is_negative * min_pos;

    vec3 left_side_times_one_over_dir  = (left_side  - ray.pos) * ray.odir;
    vec3 right_side_times_one_over_dir = (right_side - ray.pos) * ray.odir;

    t_min = max(left_side_times_one_over_dir.x, max(left_side_times_one_over_dir.y, left_side_times_one_over_dir.z));
    t_max = min(right_side_times_one_over_dir.x, min(right_side_times_one_over_dir.y, right_side_times_one_over_dir.z));

    // vec3 directionSign = sign(odir);
    // sideMin = vec3(leftSideTimesOneOverDir.x == tMin, leftSideTimesOneOverDir.y == tMin, leftSideTimesOneOverDir.z == tMin) * directionSign;
    // sideMax = vec3(rightSideTimesOneOverDir.x == tMax, rightSideTimesOneOverDir.y == tMax, rightSideTimesOneOverDir.z == tMax) * directionSign;

    return t_max > 0 && t_max > t_min;
}

Ray ray_to_model_space(Ray ray, mat4 transform) {
    vec3 new_pos = (vec4(ray.pos, 1.0) * transform).xyz;
    vec3 new_dir = (vec4(ray.dir, 0.0) * transform).xyz;

    return Ray(new_pos, new_dir, vec3(1) / new_dir);
}

#endif // _RAY_GLSL_