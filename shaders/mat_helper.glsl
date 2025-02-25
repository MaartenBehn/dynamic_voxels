#ifndef _MAT_HELPER_GLSL_
#define _MAT_HELPER_GLSL_

#define FLOAT_POS_INF uintBitsToFloat(0x7F800000)
#define FLOAT_NEG_INF uintBitsToFloat(0xFF800000)

// From
// https://github.com/yuichiroharai/glsl-y-rotate

mat4 mat4_identity() {
    return mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

mat4 mat4_scale(vec3 scale) {
    return mat4(
        scale.x, 0.0, 0.0, 0.0,
        0.0, scale.y, 0.0, 0.0,
        0.0, 0.0, scale.z, 0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

mat4 mat4_pos(vec3 pos) {
    return mat4(
    1.0, 0.0, 0.0, pos.x,
    0.0, 1.0, 0.0, pos.y,
    0.0, 0.0, 1.0, pos.z,
    0.0, 0.0, 0.0, 1.0
    );
}


mat3 mat3_rotate_x(float rad) {
    float c = cos(rad);
    float s = sin(rad);
    return mat3(
        1.0, 0.0, 0.0,
        0.0, c, s,
        0.0, -s, c
    );
}

mat3 mat3_rotate_y(float rad) {
    float c = cos(rad);
    float s = sin(rad);
    return mat3(
        c, 0.0, -s,
        0.0, 1.0, 0.0,
        s, 0.0, c
    );
}

mat3 mat3_rotate_z(float rad) {
    float c = cos(rad);
    float s = sin(rad);
    return mat3(
        c, s, 0.0,
        -s, c, 0.0,
        0.0, 0.0, 1.0
    );
}

mat3 mat3_rotate_xyz(vec3 rad) {
    mat3 rot_x = mat3_rotate_x(rad.x);
    mat3 rot_y = mat3_rotate_y(rad.y);
    mat3 rot_z = mat3_rotate_z(rad.z);

    return rot_x * rot_y * rot_z;
}

mat4 mat4_rotate_x(float rad) {
    return mat4(mat3_rotate_x(rad));
}

mat4 mat4_rotate_y(float rad) {
    return mat4(mat3_rotate_y(rad));
}

mat4 mat4_rotate_z(float rad) {
    return mat4(mat3_rotate_z(rad));
}

mat4 mat4_rotate_xyz(vec3 rad) {
    mat3 rot_x = mat3_rotate_x(rad.x);
    mat3 rot_y = mat3_rotate_y(rad.y);
    mat3 rot_z = mat3_rotate_z(rad.z);

    return mat4(rot_x * rot_y * rot_z);
}

#endif // _MAT_HELPER_GLSL_
