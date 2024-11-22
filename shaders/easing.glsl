#ifndef _EASING_GLSL_
#define _EASING_GLSL_
#define M_PI 3.1415926535897932384626433832795

float time_mod(float time, float length) {
    return mod(time * length, 1);
}

float time_mod_rot(float time, float ammount) {
    return mod(time * ammount, 2.0 * M_PI);
}

// From
// https://github.com/glslify/glsl-easings


float ease_loop(float t) {
    return t < 0.5
    ? 1.0 - t * 2.0
    : t * 2.0 - 1.0;
}

float ease_cubic_in_out(float t) {
    return t < 0.5
    ? 4.0 * t * t * t
    : 0.5 * pow(2.0 * t - 2.0, 3.0) + 1.0;
}

#endif // _EASING_GLSL_