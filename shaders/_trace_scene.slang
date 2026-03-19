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
  uint max_bounces;
  TextureHandle2D<uint8_t2> stbn_texture;  
  uint8_t* start_ptr;
  uint bvh_offset;
  uint bvh_len;
};
[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void main(uint2 pos: SV_DispatchThreadID) {
  if (pos.x == 500 && pos.y == 500) {
    //DEBUG_PRINT = pc.debug;
  }

  Scene scene = Scene(pc.start_ptr, pc.bvh_offset, pc.bvh_len);

  uint64_t clock_start = clockARB();
  BlueNoise blue_noise = BlueNoise(pc.stbn_texture.Instance, pos, pc.g_buffer.FrameNo);
  float3 ray_pos, ray_dir;
  float3 albedo;
  float emission;
  float3 irradiance;
  float3 throughput;
 
  GetPrimaryRay(pos, ray_pos, ray_dir, pc.g_buffer);

  irradiance = pc.max_bounces == 0 ? 1.0 : 0.0;
  throughput = 1.0;
 
  for (int bounce_no = 0; bounce_no <= pc.max_bounces; bounce_no++) {
    HitInfo hit = scene.ray_cast(ray_pos, ray_dir, bounce_no);
    
    if (!hit.Miss) {
      let material = pc.palette[hit.MaterialId];
      albedo = material.Color;
      emission = material.Emission;
    }

    if (bounce_no == 0) {
      if (hit.Miss) {
        pc.g_buffer.AlbedoTex.Instance[pos] = float4(1.0);
        irradiance = GetSkyColor(ray_dir, true);
        pc.g_buffer.DepthTex.Instance[pos] = -1.0;
        break;
      }
      if (pc.max_bounces == 0) {
        // fake lighting just so that faces are distinguishable
        irradiance *= hit.Normal.x != 0 ? 0.8 : 
          hit.Normal.z != 0 ? 0.5 : 
          hit.Normal.y  < 0 ? 0.3 : 1.0;
      }
      pc.g_buffer.AlbedoTex.Instance[pos] = PackAlbedo(albedo, hit.Normal, emission > 0.5);

      float4 projPos = mul(pc.g_buffer.ProjMat, float4(hit.Pos, 1.0));
      float depth = projPos.z / projPos.w;
      pc.g_buffer.DepthTex.Instance[pos] = depth;
      //pc.g_buffer.DepthTex.Instance[pos] = hit.t;
    } else {
      if (hit.Miss) {
        irradiance += GetSkyColor(ray_dir, false) * throughput;
        break;
      }
      throughput *= albedo;
    }

    irradiance += throughput * emission;

    ray_dir = normalize(hit.Normal + GetSphereDir(blue_noise.GetSample()));
    ray_pos = hit.Pos + hit.Normal * SHADER_UNIT_PER_VOXEL * 0.001;
  }

  pc.g_buffer.IrradianceTex.Instance[pos] = float4(irradiance, 0);
  pc.g_buffer.iter_tex.Instance[pos] = ITERS;
}

float3 GetSkyColor(float3 dir, bool isPrimary) {
  // For light bounces, sample smaller mips to maybe improve cache coherency.
  //return -dir;
  return float3(0.5);
  //return pc.SkyTexture.Instance.SampleLevel(g_LinearSampler, dir, isPrimary ? 0 : 3) * (isPrimary?1:4);
}

