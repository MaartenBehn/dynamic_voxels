
#extension GL_EXT_shader_realtime_clock : require
#define PROFILER_TIMING_LENGHT 1000

#include "binding.glsl"

layout(binding = 3) buffer Profiler {
    uint[] data;
} profiler;

void take_timing() {
    uint pixel_index = gl_GlobalInvocationID.x * uint(RES_X) + gl_GlobalInvocationID.y;
    uint counter_index = pixel_index * PROFILER_TIMING_LENGHT;
    uint index = counter_index + profiler.data[counter_index];

    uvec2 timing = clockRealtime2x32EXT();
    profiler.data[index + 1] = timing.x;
    profiler.data[index + 2] = timing.y;
    profiler.data[counter_index] += 2;
}

