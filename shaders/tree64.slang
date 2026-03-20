// Original Code by https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/

import helper_ray;
import helper_math;
import debug;
import perf_counters;

public struct Node {
  uint PackedData[4];

  property bool IsLeaf { get { return (PackedData[0] & 1) != 0; } }
  property uint ChildPtr { get { return PackedData[0] >> 1; } }
  property uint64_t PopMask { get { return PackedData[2] | uint64_t(PackedData[3]) << 32; } }
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
      while ((node.PopMask >> childIdx & 1) != 0 && !node.IsLeaf && scaleExp >= 2) {
        stack[scaleExp >> 1] = nodeIdx;
        nodeIdx = node.ChildPtr + popcnt_var64(node.PopMask, childIdx);
        node = nodes[nodeIdx];
        scaleExp -= 2;
        childIdx = GetNodeCellIndex(pos, scaleExp);
      }

      // Hit Leaf?
      if ((node.PopMask >> childIdx & 1) != 0 && node.IsLeaf) {
        add_iter_count(i);

        HitInfo hit;
        hit.MaterialId = leaf_data[node.ChildPtr + popcnt_var64(node.PopMask, childIdx)];
        hit.Pos = pos;

        hit.t = t_max;
        bool3 sideMask = t_max >= side_dist;
        hit.Normal = select(sideMask, -sign(dir), 0.0);
        return hit;
      }

      int advScaleExp = scaleExp;

      // Skip nodes with only one child
      if ((node.PopMask >> (childIdx & 0b101010) & 0x00330033) == 0) advScaleExp++;

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
      while ((node.PopMask >> childIdx & 1) != 0 && !node.IsLeaf) {
        stack[scaleExp >> 1] = nodeIdx;
        nodeIdx = node.ChildPtr + popcnt_var64(node.PopMask, childIdx);
        node = nodes[nodeIdx];

        scaleExp -= 2;
        childIdx = GetNodeCellIndex(pos, scaleExp) ^ mirrorMask;
      }

      // Hit Leaf?
      if ((node.PopMask >> childIdx & 1) != 0 && node.IsLeaf) {
        add_iter_count(i);

        HitInfo hit;
        hit.MaterialId = leaf_data[node.ChildPtr + popcnt_var64(node.PopMask, childIdx)];
        hit.Pos = GetMirroredPos(pos, dir, false);
        hit.t = t_max;
        bool3 sideMask = t_max >= side_dist;
        hit.Normal = select(sideMask, -sign(dir), 0.0);
        return hit;
      }

      int advScaleExp = scaleExp;

      // Skip nodes with only one child
      if ((node.PopMask >> (childIdx & 0b101010) & 0x00330033) == 0) advScaleExp++;

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

  static int GetNodeCellIndex(float3 pos, int scale_exp) {
    uint3 cellPos = asuint(pos) >> scale_exp & 3;
    return cellPos.x * 16 + cellPos.y * 4 + cellPos.z;
  }
  // floor(pos / scale) * scale
  static float3 FloorScale(float3 pos, int scale_exp) {
    uint mask = ~0u << scale_exp;
    return asfloat(asuint(pos) & mask);
  }

}
