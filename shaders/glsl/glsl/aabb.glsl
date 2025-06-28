#ifndef _AABB_GLSL_
#define _AABB_GLSL_

struct AABB {
    vec3 min;
    vec3 max;
};

bool pos_in_aabb(vec3 pos, vec3 min, vec3 max) {
    return min.x <= pos.x && pos.x <= max.x &&
        min.y <= pos.y && pos.y <= max.y &&
        min.z <= pos.z && pos.z <= max.z;
}

#endif // _AABB_GLSL_
