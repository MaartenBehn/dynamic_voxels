#ifndef _BINDING_GLSL_
#define _BINDING_GLSL_

#extension GL_EXT_scalar_block_layout : require

layout(binding = 0, rgba8) uniform writeonly image2D img;

layout(binding = 1, std430) uniform UniformBufferObject {
    vec4 pos; // w is screen_size_x
    vec4 dir; // w is screen_size_y
    vec4 data; // time in sec
} render_buffer;

#define POS render_buffer.pos.xyz
#define DIR render_buffer.dir.xyz
#define RES_X render_buffer.pos.w
#define RES_Y render_buffer.dir.w
#define RES vec2(RES_X, RES_Y)
#define TIME render_buffer.data.x;

#define MAX_CGS_TREE_SIZE 2048

layout(binding = 2, std430) buffer CGSTree {
    uint[MAX_CGS_TREE_SIZE] data;
} cgs_tree;

#define CSG_TREE cgs_tree.data

#define MAX_COLOR_BUFFER_SIZE 256
layout(binding = 3) uniform ColorBuffer {
    vec4[MAX_COLOR_BUFFER_SIZE] data;
} color_buffer;

#define COLOR_BUFFER color_buffer.data

#endif // _BINDING_GLSL_
