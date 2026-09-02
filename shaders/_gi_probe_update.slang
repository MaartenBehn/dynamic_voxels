#include "constants.h"

import texture_heap;
import helper_math;
import helper_ray;
import helper_probes;
import scene;
import tree64;

struct DispatchParams {
  ImageHandle2D<float3> radiance_atlas; 
  TextureHandle2D<float3> last_radiance_atlas; 
  ImageHandle2D<float> depth_atlas;
  TextureHandle2D<float> last_depth_atlas;
  TextureHandle2D<uint8_t2> stbn_texture; 
  Material* palette;
  uint8_t* start_ptr;
  uint active_probe_map_offset;
  uint active_probe_data_offset;
  uint frame_no;
};
[vk::push_constant] DispatchParams pc;

[shader("compute")]
[numthreads(PROBE_TILE_SIZE, PROBE_TILE_SIZE, 1)]
void main(uint probe_idx: SV_GroupID, uint2 texel_pos: SV_GroupThreadID) {
    Probe* probes = (Probe*)(pc.start_ptr + pc.active_probe_data_offset); 
    Probe probe = probes[probe_idx];

    // Compute atlas pixel coordinates
    uint2 tile = probe_tile_offset(probe_idx);
    uint2 pixel_coord = (tile * (PROBE_TILE_SIZE + PROBE_PADDING * 2)) + texel_pos + PROBE_PADDING;

    // Base texel UV in octahedral space [0, 1]
    float2 base_uv = (float2(texel_pos) + 0.5) / float(PROBE_TILE_SIZE);

    float3 accumulated_radiance = float3(0.0);
    float accumulated_depth = 0.0;

    DAG64SceneObject* tree_object = (DAG64SceneObject*)(pc.start_ptr + probe.object_offset);
    Tree64 tree = Tree64(
        (Node*)(pc.start_ptr + tree_object.node_alloc), 
        pc.start_ptr + tree_object.data_alloc, 
        tree_object.root_index
    );

    for (uint i = 0; i < RAYS_PER_TEXEL; i++) {
        uint seed = probe_idx * 73856093u + i * 19349663u + pc.frame_no;
        float2 xi = Random2(seed);

        // Jitter UV within the texel footprint for anti-aliasing / smooth direction sampling
        float2 jittered_uv = (float2(texel_pos) + xi) / float(PROBE_TILE_SIZE);
        float3 ray_dir = oct_to_dir(jittered_uv);

        HitInfo hit = tree.RayCast(probe.position, ray_dir, false);

        if (hit.miss || hit.t <= 0.0) {
            accumulated_radiance += GetSkyColor(ray_dir, true);
            accumulated_depth += 0.0;
        } else {
            let material = pc.palette[hit.MaterialId];
            accumulated_radiance += material.Color * material.Emission;
            accumulated_depth += hit.t;

            let throughput = material.Color; 

            uint16_t* active_probes_map = (uint16_t*)(pc.start_ptr + pc.active_probe_map_offset); 

            BlueNoise blue_noise = BlueNoise(pc.stbn_texture.Instance, texel_pos, pc.frame_no);
            ray_dir = normalize(hit.Normal + GetSphereDir(blue_noise.GetSample())); 
            float3 probe_irradiance = tree.sample_probe_radiance(
                hit.Pos, 
                ray_dir,
                active_probes_map,
                probes,
                pc.last_radiance_atlas,
                pc.last_depth_atlas); 
            accumulated_radiance += throughput * probe_irradiance;
        }
    }

    accumulated_radiance /= float(RAYS_PER_TEXEL);
    accumulated_depth /= float(RAYS_PER_TEXEL);

    // ---- Temporal accumulation ----
    if (pc.frame_no > 0) {
        float3 old_radiance = pc.radiance_atlas.Instance[pixel_coord];
        float old_depth = pc.depth_atlas.Instance[pixel_coord];

        float blend_alpha = 1.0 / float(pc.frame_no + 1);
        
        pc.radiance_atlas.Instance[pixel_coord] = lerp(old_radiance, accumulated_radiance, blend_alpha);
        pc.depth_atlas.Instance[pixel_coord]    = lerp(old_depth, accumulated_depth, blend_alpha);
    } else {
        pc.radiance_atlas.Instance[pixel_coord] = accumulated_radiance;
        pc.depth_atlas.Instance[pixel_coord]    = accumulated_depth;
    }
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
