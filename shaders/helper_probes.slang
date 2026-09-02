#include "constants.h"

import helper_math;
import texture_heap;

[SamplerDesc(MagFilter.Linear, MinFilter.Linear, MipFilter.Nearest, WrapMode.ClampToEdge)]
public SamplerState probe_sampler;

public struct Probe {
  public float3 position;
  public uint start_index;
  public uint object_offset;
};

public uint2 probe_tile_offset(uint atlasIndex) {
    uint x = atlasIndex % GI_ATLAS_SIZE;
    uint y = atlasIndex / GI_ATLAS_SIZE;
    return uint2(x, y);
}

public uint2 probe_pixel_coord_from_dir(uint probe_idx, float3 dir) {
  float2 uv = dir_to_oct(normalize(dir));
  uint2 tile = probe_tile_offset(probe_idx);

  float2 texel_pos = clamp(uv, 0.5 / PROBE_TILE_SIZE, 1.0 - 0.5 / PROBE_TILE_SIZE) * PROBE_TILE_SIZE;

  return (tile * (PROBE_TILE_SIZE + PROBE_PADDING * 2)) + PROBE_PADDING + uint2(texel_pos);
}

public float2 probe_uv_from_dir(uint probe_idx, float3 dir) {
  float2 uv = dir_to_oct(normalize(dir));
  uint2 tile = probe_tile_offset(probe_idx);

  float2 texel_pos = clamp(uv, 0.5 / PROBE_TILE_SIZE, 1.0 - 0.5 / PROBE_TILE_SIZE) * PROBE_TILE_SIZE;

  return (float2(tile * (PROBE_TILE_SIZE + PROBE_PADDING * 2) + PROBE_PADDING) + texel_pos) 
    / float2(GI_ATLAS_SIZE * (PROBE_TILE_SIZE + PROBE_PADDING * 2), GI_ATLAS_SIZE * (PROBE_TILE_SIZE + PROBE_PADDING * 2)); 
}

public float compute_probe_visibility(
    float dist_to_probe, 
    float sampled_probe_depth, 
    float depth_bias = 0.05
) {
    float depth_diff = dist_to_probe - sampled_probe_depth;
    return smoothstep(depth_bias, 0.0, depth_diff);
}

float compute_probe_visibility_chebyshev(float dist_to_hit, float2 sampled_moments) {
    float mean_depth = sampled_moments.x;
    float mean_depth_sq = sampled_moments.y;

    // Direct line-of-sight test (probe is closer than hit point)
    if (dist_to_hit <= mean_depth) {
        return 1.0;
    }

    // Variance evaluation: Var(D) = E[D^2] - (E[D])^2
    float variance = max(mean_depth_sq - (mean_depth * mean_depth), 0.0001);

    // Chebyshev's inequality bound
    float d = dist_to_hit - mean_depth;
    float p_max = variance / (variance + d * d);

    // Apply a smooth threshold to reduce light leaking in shadow corners
    float weight = smoothstep(0.2, 1.0, p_max);
    return weight;
}
