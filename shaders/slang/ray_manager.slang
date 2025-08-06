import tree64;
import g_buffer;
import texture_heap;
import helper_ray;
import helper_math;
import debug;

#include "perf_counters.h.slang"

public struct RayManager {
  public GBuffer* g_buffer;
  public Material* palette;
  public PerfStats* perf_counters;
  public uint max_bounces;
  public TextureHandle2D<uint8_t2> stbn_texture;  
}

public struct RayRunner {
  public BlueNoise blue_noise; 
  public uint64_t clock_start;
  public float3 ray_pos, ray_dir; 
  public float3 irradiance;
  public float3 throughput;

  public __init(uint2 pos, RayManager rm) {
    clock_start = clockARB();
    blue_noise = BlueNoise(rm.stbn_texture.Instance, pos, rm.g_buffer.FrameNo);

    GetPrimaryRay(pos, ray_pos, ray_dir, rm.g_buffer);

    irradiance = rm.max_bounces == 0 ? 1.0 : 0.0;
    throughput = 1.0;
  }

  public void prepare_trace() {
    PERF_STAT_INC(RayCasts, 1);
    PERF_TIME_BEGIN(ClocksPerRay);
  }
  
  [mutating]
  public bool after_trace(uint2 pos, HitInfo hit, RayManager rm, uint bounce_no) {
    PERF_TIME_END(ClocksPerRay);

    float3 albedo;
    float emission;

    if (!hit.Miss) {
      let material = rm.palette[hit.MaterialId];
      albedo = material.Color;
      emission = material.Emission;
    }

    if (bounce_no == 0) {
      if (hit.Miss) {
        rm.g_buffer.AlbedoTex.Instance[pos] = float4(1.0);
        irradiance = GetSkyColor(ray_dir, true);
        rm.g_buffer.DepthTex.Instance[pos] = -1.0;
        return false;
      }
      if (rm.max_bounces == 0) {
        // fake lighting just so that faces are distinguishable
        irradiance *= hit.Normal.x != 0 ? 0.8 : 
          hit.Normal.z != 0 ? 0.5 : 
          hit.Normal.y  < 0 ? 0.3 : 1.0;
      }
      rm.g_buffer.AlbedoTex.Instance[pos] = PackAlbedo(albedo, hit.Normal, emission > 0.5);

      float4 projPos = mul(rm.g_buffer.ProjMat, float4(hit.Pos, 1.0));
      float depth = projPos.z / projPos.w;
      rm.g_buffer.DepthTex.Instance[pos] = depth;
      //rm.g_buffer.DepthTex.Instance[pos] = hit.t;
    } else {
      if (hit.Miss) {
        irradiance += GetSkyColor(ray_dir, false) * throughput;
        return false;
      }
      throughput *= albedo;
    }

    irradiance += throughput * emission;

    ray_dir = normalize(hit.Normal + GetSphereDir(blue_noise.GetSample()));
    ray_pos = hit.Pos + hit.Normal * SHADER_UNIT_PER_VOXEL * 0.1;

    return true;
  }

  public void finish(uint2 pos, RayManager rm) {
    uint64_t clock_end = clockARB();
    float clockDiff = (clock_end - clock_start) / 1000.0;

#if PERF_PRECISE_ITER_HEATMAP
  clockDiff = g_CurrTraversalIters;
#endif

    rm.g_buffer.IrradianceTex.Instance[pos] = float4(irradiance, clockDiff);
  }
}

//[vk::binding(0, 1)]
//[SamplerDesc(MagFilter.Linear, MinFilter.Linear, MipFilter.Linear, WrapMode.Repeat)]
//public SamplerState g_LinearSampler;

float3 GetSkyColor(float3 dir, bool isPrimary) {
  // For light bounces, sample smaller mips to maybe improve cache coherency.
  //return -dir;
  return float3(0.5);
  //return pc.SkyTexture.Instance.SampleLevel(g_LinearSampler, dir, isPrimary ? 0 : 3) * (isPrimary?1:4);
}


