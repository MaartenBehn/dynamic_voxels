#include "constants.h"

import texture_heap;
import helper_math;
import helper_ray;
import scene;
import tree64;
import probe;

struct DispatchParams {
  ImageHandle2D<float3> radiance_atlas; 
  ImageHandle2D<float> depth_atlas;
  Material* palette;
  uint8_t* start_ptr;
  uint probes_offset;
};
[vk::push_constant] DispatchParams pc;

[shader("compute")]
[numthreads(PROBE_TILE_SIZE, PROBE_TILE_SIZE, 1)]
void main(uint probeIdx: SV_GroupID, uint2 texel_pos: SV_GroupThreadID) {
  uint frameIndex = 0; 

  Probe* probes = (Probe*)(pc.start_ptr + pc.probes_offset); 
  Probe probe = probes[probeIdx];

  // ---- Compute atlas coord ----
  uint2 tile = ProbeTileOffset(probeIdx, GI_ATLAS_SIZE);

  uint2 pixelCoord = (tile * (PROBE_TILE_SIZE + PROBE_PADDING * 2)) + texel_pos + PROBE_PADDING;

  // ---- Convert texel → direction ----
  float2 uv = (texel_pos + 0.5) / PROBE_TILE_SIZE;
  float3 dir = oct_to_dir(uv);

  float3 radiance = float3(0,0,0);
  float depth = 0.0;

  DAG64SceneObject* tree_object = (DAG64SceneObject*)(pc.start_ptr + probe.object_offset);
  Tree64 tree = Tree64((Node*)(pc.start_ptr + tree_object.node_alloc), pc.start_ptr + tree_object.data_alloc, tree_object.root_index);

  for (uint i = 0; i < RAYS_PER_TEXEL; i++) {
    uint seed = probeIdx * 73856093u + i * 19349663u + frameIndex;

    float2 xi = Random2(seed);

    float3 local = CosineSampleHemisphere(xi);
    float3 rayDir = ToWorld(local, dir);

    HitInfo hit = tree.RayCast(probe.position, rayDir, false);

    if (hit.miss) {
      radiance += GetSkyColor(rayDir, true);
      depth = -1.0;
    } else {
      let material = pc.palette[hit.MaterialId];
      radiance += material.Color * material.Emission;
      depth += hit.t;
    }
  }

  radiance /= RAYS_PER_TEXEL;
  depth /= RAYS_PER_TEXEL;

  // ---- Temporal accumulation ----
  float3 oldRad = pc.radiance_atlas.Instance[pixelCoord];
  float oldDepth = pc.depth_atlas.Instance[pixelCoord];

  float blendAlpha = 1.0;
  pc.radiance_atlas.Instance[pixelCoord] = lerp(oldRad, radiance, blendAlpha);
  pc.depth_atlas.Instance[pixelCoord]    = lerp(oldDepth, depth, blendAlpha);

}


uint2 ProbeTileOffset(uint atlasIndex, uint atlasWidthInTiles) {
    uint x = atlasIndex % atlasWidthInTiles;
    uint y = atlasIndex / atlasWidthInTiles;
    return uint2(x, y);
}

float Hash(uint x) {
    x ^= x >> 17;
    x *= 0xed5ad4bb;
    x ^= x >> 11;
    x *= 0xac4c1b51;
    x ^= x >> 15;
    x *= 0x31848bab;
    x ^= x >> 14;
    return frac(x / 65536.0);
}

float2 Random2(uint seed) {
    return float2(Hash(seed), Hash(seed * 1664525u));
}

float3 CosineSampleHemisphere(float2 xi) {
    float r = sqrt(xi.x);
    float theta = 2.0 * 3.14159265 * xi.y;

    float x = r * cos(theta);
    float y = r * sin(theta);
    float z = sqrt(1.0 - xi.x);

    return float3(x, y, z);
}

void BuildBasis(float3 n, out float3 t, out float3 b) {
    float3 up = abs(n.z) < 0.999 ? float3(0,0,1) : float3(1,0,0);
    t = normalize(cross(up, n));
    b = cross(n, t);
}

float3 ToWorld(float3 localDir, float3 n) {
    float3 t, b;
    BuildBasis(n, t, b);
    return localDir.x * t + localDir.y * b + localDir.z * n;
}
