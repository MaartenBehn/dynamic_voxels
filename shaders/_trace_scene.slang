import scene;
import helper_ray;
import helper_math;
import texture_heap;
import debug;
import g_buffer;
import perf_counters;

public struct DispatchParams {
  GBuffer* g_buffer;
  Material* palette;
  TextureHandle2D<uint8_t2> stbn_texture;  
  uint8_t* start_ptr;
  uint bvh_offset;
  uint bvh_len;
  uint max_bounces;
};
[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void main(uint2 pos: SV_DispatchThreadID) {
  //if (pc.max_bounces > 0) {
    path_trace_scene(pos);
    return;
  //}

  //Scene scene = Scene(pc.start_ptr, pc.bvh_offset, pc.bvh_len);


}

void path_trace_scene(uint2 pos) {
  if (pos.x == 500 && pos.y == 500) {
    DEBUG_PRINT = true;
  }

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

  for (int bounce_no = 1; bounce_no <= pc.max_bounces; bounce_no++) {
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

  pc.g_buffer.IrradianceTex.Instance[pos] = float4(irradiance, 0);
  pc.g_buffer.iter_tex.Instance[pos] = ITERS;
}
