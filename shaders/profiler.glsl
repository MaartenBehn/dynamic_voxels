
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

void profile_scope_begin(uint id) {
    uint pixel_index = gl_GlobalInvocationID.x * uint(RES_X) + gl_GlobalInvocationID.y;
    if (pixel_index != 0) {
        return;
    }
    uint index = id * 4;

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
    uint index = id * 4;

    float counter = float(profiler_out.data[index]);
    uint64_t start = uint64_t(profiler_out.data[index + 1]) + uint64_t(profiler_out.data[index + 2] << 32);
    uint64_t end = clockRealtimeEXT();
    float new = float(end - start);
    float old = uintBitsToFloat(profiler_out.data[index + 3]);
    float mean = (old * (counter - 1.0) + new) / counter;
    profiler_out.data[index + 3] = floatBitsToInt(mean);
}

