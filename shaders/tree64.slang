// Original Code by https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/

#include "constants.h"

import helper_ray;
import helper_math;
import helper_probes;
import debug;
import perf_counters;
import texture_heap;

public struct Node {
  uint packed_data[4];

  public property bool IsLeaf { get { return (packed_data[0] & 1) != 0; } }
  public property uint ChildPtr { get { return packed_data[0] >> 1; } }
  public property uint gi_probe_index { get { return packed_data[1]; } }
  public property uint64_t pop_mask { get { return packed_data[2] | uint64_t(packed_data[3]) << 32; } }
}

public struct Tree64 {
  Node* nodes;
  uint8_t* leaf_data;
  uint start_index;

  public __init(Node* nodes, uint8_t* leaf_data, uint start_index) {
    this.nodes = nodes;
    this.leaf_data = leaf_data;
    this.start_index = start_index;
  }

  public HitInfo RayCast(float3 rayPos, float3 rayDir, bool coarse) {
    HitInfo hit = Traversal(rayPos, rayDir, coarse);
    return hit;
  }

  float3 non_align(float3 dir) {

    float epsilon =  0.0001; 
    if (abs(dir.x) < epsilon) dir.x = epsilon;
    if (abs(dir.y) < epsilon) dir.y = epsilon;
    if (abs(dir.z) < epsilon) dir.z = epsilon;
    return dir;
  }

  HitInfo Traversal(float3 origin, float3 dir, bool coarse) {
    float3 inv_dir = 1.0f / dir;

    float3 side_dist, pos;
    float t_min, t_max;
    uint stack[11];
    int scaleExp = 21;  // 0.25

    if (any(origin < 1.0 || origin >= 2.0)) {

      float3 is_positive = float3(float(inv_dir.x > 0), float(inv_dir.y > 0), float(inv_dir.z >= 0));     
      float3 is_negative = 1.0f - is_positive;

      float3 min_pos = 1.0001;
      float3 max_pos = 1.9999;

      float3 min_side = is_positive * min_pos + is_negative * max_pos;
      float3 max_side = is_positive * max_pos + is_negative * min_pos;

      side_dist = (min_side - origin) * inv_dir;
      float3 out_side_dist = (max_side - origin) * inv_dir;

      t_min = max(side_dist.x, max(side_dist.y, side_dist.z));
      t_max = min(out_side_dist.x, min(out_side_dist.y, out_side_dist.z));

      if (t_max < t_min) {
        HitInfo hit;
        return hit; 
      }
      pos = origin + dir * t_min;
    } else {
      pos = origin;
    }

    uint nodeIdx = start_index; // root
    Node node = nodes[nodeIdx];

    if (node.IsLeaf) {
        HitInfo hit;
        hit.t = t_min;
        hit.Pos = pos;
        bool3 sideMask = t_min <= side_dist;
        hit.Normal = select(sideMask, -sign(dir), 0.0);
        return hit;
    }

    int i;
    for (i = 0; i < 256; i++) {
      int childIdx = GetNodeCellIndex(pos, scaleExp);

      // Descend
      while ((node.pop_mask >> childIdx & 1) != 0 && !node.IsLeaf && scaleExp >= 2) {
        stack[scaleExp >> 1] = nodeIdx;
        nodeIdx = node.ChildPtr + popcnt_var64(node.pop_mask, childIdx);
        node = nodes[nodeIdx];
        scaleExp -= 2;
        childIdx = GetNodeCellIndex(pos, scaleExp);
      }

      // Hit Leaf?
      if ((node.pop_mask >> childIdx & 1) != 0 && node.IsLeaf) {
        add_iter_count(i);

        HitInfo hit;
        hit.MaterialId = leaf_data[node.ChildPtr + popcnt_var64(node.pop_mask, childIdx)];
        hit.Pos = pos;

        hit.t = t_max;
        bool3 sideMask = t_max >= side_dist;
        hit.Normal = select(sideMask, -sign(dir), 0.0);
        return hit;
      }

      int advScaleExp = scaleExp;

      // Skip nodes with only one child
      if ((node.pop_mask >> (childIdx & 0b101010) & 0x00330033) == 0) advScaleExp++;

      // Compute next pos by intersecting with max cube edges
      pos = FloorScale(pos, advScaleExp);
      float3 prev_pos = pos;

      float scale = bit_cast<float>((advScaleExp - 23 + 127) << 23);  // 2 ^ -(23 - advScaleExp)
      side_dist = (step(0.0f, dir) * scale + (pos - origin)) * inv_dir;

      // Bounds clamping      
      t_max = min(min(side_dist.x, side_dist.y), side_dist.z);

      float3 sibl_pos_0 = select(t_max == side_dist, pos + copysign(scale, dir), pos);
      float3 sibl_pos_1 = asfloat(bit_cast<int3>(sibl_pos_0) + ((1 << advScaleExp) - 1));
      pos = clamp(origin + dir * t_max, sibl_pos_0, sibl_pos_1); // clamp to sibling bounds [0, 1)

      // Find carry bit to tell which node we should ascend to
      uint3 diffPos = asuint(pos) ^ asuint(prev_pos);
      int diffExp = firstbithigh(diffPos.x | diffPos.y | diffPos.z); // 31 - lzcnt
      if (diffExp % 2 == 0) diffExp--;

      if (diffExp > scaleExp) {
        scaleExp = diffExp;
        if (diffExp > 21) break;  // going out of root?

        scaleExp = diffExp;
        nodeIdx = stack[scaleExp >> 1];
        node = nodes[nodeIdx];
      }
    }
    add_iter_count(i);

    HitInfo hit;
    return hit;
  }

  HitInfo Traversal_OctMirror(float3 origin, float3 dir, bool coarse) {
    float3 side_dist, pos;
    float t_min, t_max;

    float3 inv_dir = 1.0f / dir;

    if (any(origin < 1.0 || origin >= 2.0)) {

      float3 is_positive = float3(float(inv_dir.x > 0), float(inv_dir.y > 0), float(inv_dir.z >= 0));     
      float3 is_negative = 1.0f - is_positive;

      float3 min_pos = 1.0001;
      float3 max_pos = 1.9999;

      float3 min_side = is_positive * min_pos + is_negative * max_pos;
      float3 max_side = is_positive * max_pos + is_negative * min_pos;

      side_dist = (min_side - origin) * inv_dir;
      float3 out_side_dist = (max_side - origin) * inv_dir;

      t_min = max(side_dist.x, max(side_dist.y, side_dist.z));
      t_max = min(out_side_dist.x, min(out_side_dist.y, out_side_dist.z));

      if (t_max < t_min) {
        HitInfo hit;
        return hit; 
      }

      pos = GetMirroredPos(origin + dir * t_min, dir, false);
      origin = GetMirroredPos(origin, dir, false);
    } else {
      origin = GetMirroredPos(origin, dir, false);
      pos = origin;
    }

    uint nodeIdx = start_index; // root
    Node node = nodes[nodeIdx];

    if (node.IsLeaf) {
        HitInfo hit;
        hit.t = t_min;
        bool3 sideMask = t_min <= side_dist;
        hit.Normal = select(sideMask, -sign(dir), 0.0);
        return hit;
    }

    uint groupId = spirv_asm {
      OpCapability VariablePointers;
      result:$$uint = OpLoad builtin(LocalInvocationIndex:uint);
    };

    static groupshared uint gs_stack[64][11];
    uint* stack = &gs_stack[groupId][0];
    int scaleExp = 21;  // 0.25
    
    // Mirror coordinates to negative ray octant
    uint mirrorMask = 0;
    if (dir.x > 0) mirrorMask |= 3 << 4;
    if (dir.y > 0) mirrorMask |= 3 << 2;
    if (dir.z > 0) mirrorMask |= 3 << 0;
    
    // Mirror inv_dir
    inv_dir = -abs(inv_dir);
    
    int childIdx;
    int i;
    for (i = 0; i < 256; i++) {
      childIdx = GetNodeCellIndex(pos, scaleExp) ^ mirrorMask;

      // Descend
      while ((node.pop_mask >> childIdx & 1) != 0 && !node.IsLeaf) {
        stack[scaleExp >> 1] = nodeIdx;
        nodeIdx = node.ChildPtr + popcnt_var64(node.pop_mask, childIdx);
        node = nodes[nodeIdx];

        scaleExp -= 2;
        childIdx = GetNodeCellIndex(pos, scaleExp) ^ mirrorMask;
      }

      // Hit Leaf?
      if ((node.pop_mask >> childIdx & 1) != 0 && node.IsLeaf) {
        add_iter_count(i);

        HitInfo hit;
        hit.MaterialId = leaf_data[node.ChildPtr + popcnt_var64(node.pop_mask, childIdx)];
        hit.Pos = GetMirroredPos(pos, dir, false);
        hit.t = t_max;
        bool3 sideMask = t_max >= side_dist;
        hit.Normal = select(sideMask, -sign(dir), 0.0);
        return hit;
      }

      int advScaleExp = scaleExp;

      // Skip nodes with only one child
      if ((node.pop_mask >> (childIdx & 0b101010) & 0x00330033) == 0) advScaleExp++;

      // Compute next pos by intersecting with max cube edges
      float3 edgePos = FloorScale(pos, advScaleExp);

      side_dist = (edgePos - origin) * inv_dir;
      t_max = min(min(side_dist.x, side_dist.y), side_dist.z);

      int3 maxSiblBounds = bit_cast<int3>(edgePos) + select(side_dist == t_max, -1, (1 << advScaleExp) - 1);
      pos = min(origin - abs(dir) * t_max, asfloat(maxSiblBounds));

      // Find carry bit to tell which node we should ascend to
      uint3 diffPos = asuint(pos) ^ asuint(edgePos);
      int diffExp = firstbithigh(diffPos.x | diffPos.y | diffPos.z); // 31 - lzcnt
      if (diffExp % 2 == 0) diffExp--;

      if (diffExp > scaleExp) {
        scaleExp = diffExp;
        if (diffExp > 21) break;  // going out of root?

        nodeIdx = stack[scaleExp >> 1];
        node = nodes[nodeIdx];
      }
    }
    add_iter_count(i);

    HitInfo hit;
    return hit;
  }

  // Computes `dir > 0 ? 3.0 - pos : pos`
  // with special case to avoid rounding.
  static float3 GetMirroredPos(float3 pos, float3 dir, bool outside) {
    float3 mirrored = asfloat(asuint(pos) ^ 0x7FFFFF);
    // XOR'ing will only work if the coords are in range [1.0, 2.0),
    // so just fallback to the original subtraction if that's not the case.
    if (outside) mirrored = 3.0 - pos;
    return select(dir > 0, mirrored, pos);
  }

    // floor(pos / scale) * scale
  static float3 FloorScale(float3 pos, int scale_exp) {
    uint mask = ~0u << scale_exp;
    return asfloat(asuint(pos) & mask);
  }

  public float3 sample_probe_radiance(
      float3 pos,
      float3 dir,
      uint16_t* active_probes_map,
      Probe* probes,
      TextureHandle2D<float3> radiance_atlas,
      TextureHandle2D<float> depth_atlas,
      float probe_depth_bias,
      int debug_level = -1
      ) {
    uint node_idx = start_index;
    Node node = nodes[node_idx];

    float3 accumulated_radiance = float3(0.0, 0.0, 0.0);
    float total_weight = 0.0;

    int scale_exp = 21;
    int current_level = 0;

    for (int i = 0; i < 6; i++) {
        if (debug_level < 0 || debug_level == current_level) {
            
            // Fast bitcast equivalent to: float cell_size = exp2((float)(scale_exp - 23));
            float cell_size = bit_cast<float>((scale_exp - 23 + 127) << 23); 
            
            // Map position to a continuous cell index space centered on probes
            float3 normalized_pos = pos - 1.0f; // Shift to [0, 1) domain
            float3 continuous_idx = (normalized_pos / cell_size) - 0.5f;

            int3 base_idx = int3(floor(continuous_idx));
            float3 f = frac(continuous_idx);

            float3 level_radiance = float3(0.0, 0.0, 0.0);
            float level_vis_weight = 0.0;

            // Sample the 8 trilinear neighborhood corners
            [unroll]
            for (int corner = 0; corner < 8; corner++) {
                int3 offset = int3(corner & 1, (corner >> 1) & 1, (corner >> 2) & 1);
                
                // Trilinear weight for this corner
                float trilinear_w = (offset.x ? f.x : (1.0 - f.x)) *
                                    (offset.y ? f.y : (1.0 - f.y)) *
                                    (offset.z ? f.z : (1.0 - f.z));

                // Quick skip if trilinear weight is negligible
                if (trilinear_w < 0.001) continue;

                int3 neighbor_idx = base_idx + offset;
                float3 neighbor_pos = (neighbor_idx + 0.5f) * cell_size + 1.0f; // Map back to world pos
                
                uint active_probe_index = fetch_probe_global(neighbor_pos, scale_exp, active_probes_map);

                if (active_probe_index != ACTIVE_PROBE_INDEX_NONE) {
                    Probe probe = probes[active_probe_index];

                    float3 probe_to_hit = pos - probe.position;
                    float dist_to_hit = length(probe_to_hit);
                    float3 dir_to_hit = probe_to_hit / dist_to_hit;

                    float2 depth_uv = probe_uv_from_dir((uint)active_probe_index, dir_to_hit);
                    float probe_depth = depth_atlas.Instance.SampleLevel(probe_sampler, depth_uv, 0.0).x;
                    
                    float visibility_weight = compute_probe_visibility(dist_to_hit, probe_depth, probe_depth_bias);
                    float combined_weight = trilinear_w * visibility_weight;

                    if (combined_weight > 0.001) {
                        float2 radiance_uv = probe_uv_from_dir((uint)active_probe_index, dir);
                        float3 probe_radiance = radiance_atlas.Instance.SampleLevel(probe_sampler, radiance_uv, 0.0).rgb;

                        level_radiance += probe_radiance * combined_weight;
                        level_vis_weight += combined_weight;
                    }
                }
            }

            // Accumulate layer contribution to global output
            if (level_vis_weight > 0.001) {
                float level_weight_factor = pow(2.0, (float)current_level);
                
                // Normalize layer radiance to prevent darkening at sparse boundaries,
                // then scale by the layer's overall validity/weight.
                float3 normalized_layer_radiance = level_radiance / level_vis_weight;
                float final_layer_weight = level_weight_factor * min(level_vis_weight, 1.0);

                accumulated_radiance += normalized_layer_radiance * final_layer_weight;
                total_weight += final_layer_weight;
            }

            if (debug_level == current_level) {
                break;
            }
        }

        // Descend main query point to next hierarchy level
        int child_idx = GetNodeCellIndex(pos, scale_exp);
        if ((node.pop_mask >> child_idx & 1) != 0 && !node.IsLeaf && scale_exp >= 2) {
            scale_exp -= 2;
            node_idx = node.ChildPtr + popcnt_var64(node.pop_mask, child_idx);
            node = nodes[node_idx];
            current_level++;
        } else {
            break;
        }
    }

    if (total_weight > 0.0) {
        if (debug_level >= 0) {
            return accumulated_radiance;
        }
        return accumulated_radiance / total_weight;
    }

    return float3(0.0, 0.0, 0.0);
}

  uint fetch_probe_global(
      float3 target_pos, 
      int target_scale_exp, 
      uint16_t* active_probes_map) {

    // Bounds check based on your traversal boundaries [1.0, 2.0)
    if (any(target_pos < 1.0f) || any(target_pos >= 2.0f)) {
      return ACTIVE_PROBE_INDEX_NONE;
    }

    uint node_idx = start_index;
    Node node = nodes[node_idx];
    int current_scale = 21; // Root scale

    while (current_scale > target_scale_exp) {
      int child_idx = GetNodeCellIndex(target_pos, current_scale);

      // If empty space or leaf is reached before target scale, no probe exists here
      if ((node.pop_mask >> child_idx & 1) == 0 || node.IsLeaf) {
        return ACTIVE_PROBE_INDEX_NONE;
      }

      current_scale -= 2;
      node_idx = node.ChildPtr + popcnt_var64(node.pop_mask, child_idx);
      node = nodes[node_idx];
    }

    // We reached the target scale level. Check if a probe exists.
    if (current_scale == target_scale_exp && node.gi_probe_index != GI_PROBE_INDEX_NONE) {
      return active_probes_map[node.gi_probe_index];
    }

    return ACTIVE_PROBE_INDEX_NONE;
  }
}

public static uint GetNodeCellIndex(float3 pos, uint scale_exp) {
  uint3 cellPos = asuint(pos) >> scale_exp & 3;
  return cellPos.x * 16 + cellPos.y * 4 + cellPos.z;
}



