#include "constants.h"

import helper_math;

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
  return (tile * (PROBE_TILE_SIZE + PROBE_PADDING * 2)) 
                     + uint2(texel_pos) 
                     + PROBE_PADDING;
}

public float2 probe_uv_from_dir(uint probe_idx, float3 dir) {
    float2 oct_uv = dir_to_oct(dir); // Maps 3D vector to [0, 1] octahedral square

    // Map octahedral square into the probe's specific tile footprint in the atlas
    float2 tile_size = float2(PROBE_TILE_SIZE);
    float2 atlas_size = float2(GI_ATLAS_SIZE, GI_ATLAS_SIZE);
    
    uint2 tile_idx = probe_tile_offset(probe_idx);
    float2 tile_min_pixel = float2(tile_idx * (PROBE_TILE_SIZE + 2) + 1);

    // Position within pixel centers
    float2 pixel_coord = tile_min_pixel + oct_uv * tile_size;
    return pixel_coord / atlas_size;
}

public float compute_probe_visibility(
    float dist_to_probe, 
    float sampled_probe_depth, 
    float depth_bias = 0.05
) {
    if (sampled_probe_depth <= 0.0) {
        return 1.0;
    }

    // If the hit point is further from the probe than the probe's recorded distance to geometry,
    // a surface sits between the probe and our hit point.
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
