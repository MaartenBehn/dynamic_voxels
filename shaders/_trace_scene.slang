#include "constants.h"

import scene;
import helper_ray;
import helper_math;
import helper_probes;
import texture_heap;
import debug;
import g_buffer;
import perf_counters;

public struct DispatchParams {
  float3 debug_probe_pos; 
  uint debug_probe_index;

  GBuffer* g_buffer;
  Material* palette;
  TextureHandle2D<uint8_t2> stbn_texture; 
  TextureHandle2D<float3> radiance_atlas; 
  TextureHandle2D<float> depth_atlas;
  uint8_t* start_ptr;
  uint bvh_offset;
  uint bvh_len;
  uint active_probe_map_offset;
  uint active_probe_data_offset;
  uint max_bounces; 
  uint use_probes;
  uint debug_probe_depth;
  int only_use_probe_level;
};
[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void main(uint2 pos: SV_DispatchThreadID) {
  //if (pos.x == 500 && pos.y == 500) {
  //  DEBUG_PRINT = true;
  //}
  
  path_trace_scene(pos); return;

  Scene scene = Scene(pc.start_ptr, pc.bvh_offset, pc.bvh_len);

  float3 ray_pos, ray_dir;
  GetPrimaryRay(pos, ray_pos, ray_dir, pc.g_buffer);
  ObjectHitInfo hit = scene.ray_cast(ray_pos, ray_dir);

  if (hit.miss) {
    pc.g_buffer.AlbedoTex.Instance[pos] = float4(1.0);
    pc.g_buffer.IrradianceTex.Instance[pos] = float4(GetSkyColor(ray_dir, true), 0);
    pc.g_buffer.DepthTex.Instance[pos] = -1.0;
    pc.g_buffer.iter_tex.Instance[pos] = ITERS;
    return;
  }

  float3 gloabl_pos = hit.global_pos();
  float4 projPos = mul(pc.g_buffer.ProjMat, float4(gloabl_pos, 1.0));
  float depth = projPos.z / projPos.w;
  pc.g_buffer.DepthTex.Instance[pos] = depth;

  let material = pc.palette[hit.material_id];
  float3 albedo = material.Color;
  float emission = material.Emission;
  pc.g_buffer.AlbedoTex.Instance[pos] = PackAlbedo(albedo, hit.normal, emission > 0.5);  
}

void path_trace_scene(uint2 pos) {
  Scene scene = Scene(pc.start_ptr, pc.bvh_offset, pc.bvh_len);

  float3 ray_pos, ray_dir;
  GetPrimaryRay(pos, ray_pos, ray_dir, pc.g_buffer);
  ObjectHitInfo hit = scene.ray_cast(ray_pos, ray_dir);
    
  if (hit.miss) {
  pc.g_buffer.AlbedoTex.Instance[pos] = float4(1.0);
    pc.g_buffer.IrradianceTex.Instance[pos] = float4(GetSkyColor(ray_dir, true), 0);
    pc.g_buffer.DepthTex.Instance[pos] = -1.0;
    pc.g_buffer.iter_tex.Instance[pos] = ITERS;
    return;
  }

  float3 gloabl_pos = hit.global_pos();
  float4 projPos = mul(pc.g_buffer.ProjMat, float4(gloabl_pos, 1.0));
  float depth = projPos.z / projPos.w;
  pc.g_buffer.DepthTex.Instance[pos] = depth;

  if (hit.material_id == MATERIAL_ID_PROBE) {
    float3 incomingLightDir = normalize(pc.debug_probe_pos - gloabl_pos);
    uint2 coord = probe_pixel_coord_from_dir(pc.debug_probe_index, -incomingLightDir);
    
    if (pc.debug_probe_depth != 0) {
      pc.g_buffer.IrradianceTex.Instance[pos] = float4(float3(pc.depth_atlas.Instance[coord]), 0);
    } else {
      pc.g_buffer.IrradianceTex.Instance[pos] = float4(float3(pc.radiance_atlas.Instance[coord]), 0);
    }

    pc.g_buffer.iter_tex.Instance[pos] = ITERS;
    return;
  }

  let material = pc.palette[hit.material_id];
  float3 albedo = material.Color;
  float emission = material.Emission;
  pc.g_buffer.AlbedoTex.Instance[pos] = PackAlbedo(albedo, hit.normal, emission > 0.5);
 
  if (pc.max_bounces == 0) {
    // fake lighting just so that faces are distinguishable
    float3 irradiance = hit.normal.x != 0 ? 0.8 : 
      hit.normal.z != 0 ? 0.5 : 
      hit.normal.y  < 0 ? 0.3 : 1.0; 
    pc.g_buffer.IrradianceTex.Instance[pos] = float4(irradiance, 0);

    pc.g_buffer.iter_tex.Instance[pos] = ITERS;
    return;
  }  

  float3 throughput = albedo;
  float3 irradiance = throughput * emission;
 
  BlueNoise blue_noise = BlueNoise(pc.stbn_texture.Instance, pos, pc.g_buffer.FrameNo);

  for (int bounce_no = 2; bounce_no <= pc.max_bounces; bounce_no++) {
    hit.object.ray_dir = normalize(hit.normal + GetSphereDir(blue_noise.GetSample()));
    hit.object.pos = hit.object.pos + hit.normal * (1.0 / hit.object.object.size);

    hit = scene.ray_cast_from(hit.object);

    if (hit.miss) {
      irradiance += GetSkyColor(ray_dir, false) * throughput;
      break;
    }

    let material = pc.palette[hit.material_id];
    float3 albedo = material.Color;
    float emission = material.Emission;
 
    throughput *= albedo;
    irradiance += throughput * emission;
  }

  if (pc.use_probes != 0) {
    uint16_t* active_probes_map = (uint16_t*)(pc.start_ptr + pc.active_probe_map_offset); 
    Probe* probes = (Probe*)(pc.start_ptr + pc.active_probe_data_offset); 
    
    let dir = normalize(hit.normal + GetSphereDir(blue_noise.GetSample())); 
    float3 probe_irradiance = hit.object.tree.sample_probe_radiance(
      hit.object.pos, 
      dir,
      active_probes_map,
      probes,
      pc.radiance_atlas,
      pc.depth_atlas,
      pc.only_use_probe_level); 
    irradiance += throughput * probe_irradiance;
  }

  pc.g_buffer.IrradianceTex.Instance[pos] = float4(irradiance, 0);
  pc.g_buffer.iter_tex.Instance[pos] = ITERS;
}



