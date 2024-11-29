
#extension GL_EXT_shader_realtime_clock : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

#include "binding.glsl"

layout(binding = 10) uniform ProfilerIn {
    uint active_pixel;
    uint max_timings;
    uint mode;
} profiler_in;

layout(binding = 11) buffer ProfilerOut {
    uint[] data;
} profiler_out;

void init_profile(){
    uint pixel_index = gl_GlobalInvocationID.x * uint(RES_X) + gl_GlobalInvocationID.y;
    if (pixel_index != 0) {
        return;
    }

    for (uint i = 0; i < 30; i++) {
        profiler_out.data[i * 5] = 0;
    }
}

void profile_scope_begin(uint id) {
    uint pixel_index = gl_GlobalInvocationID.x * uint(RES_X) + gl_GlobalInvocationID.y;
    if (pixel_index != 0) {
        return;
    }
    uint index = id * 5;

    uint64_t timing = clockRealtimeEXT();
    profiler_out.data[index]++;
    profiler_out.data[index + 1] = uint(timing);
    profiler_out.data[index + 2] = uint(timing >> 32);
}

void profile_scope_end(uint id) {
    uint pixel_index = gl_GlobalInvocationID.x * uint(RES_X) + gl_GlobalInvocationID.y;
    if (pixel_index != 0) {
        return;
    }
    uint index = id * 5;

    uint64_t end = clockRealtimeEXT();
    profiler_out.data[index + 3] = uint(end);
    profiler_out.data[index + 4] = uint(end >> 32);
}

