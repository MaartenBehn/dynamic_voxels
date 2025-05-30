import backend_common;
import utilits;
import ray;
import tree64;
import g_buffer;

#include "perf_counters.h.slang"

static uint MaxBounces = 0;
static int3 WorldOrigin = int3(0, 0, 0);
static uint NumSteadyFrames = 0;

[vk::binding(0, 1)] public Texture2D<uint2> StbnTexture;              

public struct DispatchParams {
  VoxelTree tree;
  GBufferUniforms* GBuffer;
  Material* Palette;
  PerfStats* PerfCounters;
  uint MaxBounces;
};

[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {

  uint64_t clockStart = clockARB();

  float3 rayDir, rayPos;
  GetPrimaryRay(pos, rayPos, rayDir);

  var blueNoise = BlueNoise(StbnTexture, pos, pc.GBuffer->FrameNo);

  float3 irradiance = pc.MaxBounces == 0 ? 1.0 : 0.0;
  float3 throughput = 1.0;

  for (int bounceNo = 0; bounceNo <= pc.MaxBounces; bounceNo++) {
    PERF_STAT_INC(RayCasts, 1);
    PERF_TIME_BEGIN(ClocksPerRay);
    HitInfo hit = pc.tree.RayCast(pc.GBuffer.position, rayPos, rayDir, bounceNo != 0);
    PERF_TIME_END(ClocksPerRay);

    float3 albedo;
    float emission;

    if (!hit.Miss) {
      let material = pc.Palette[hit.MaterialId];
      albedo = material.Color;
      emission = material.Emission;
    }

    if (bounceNo == 0) {
      if (hit.Miss) {
        AlbedoTex[pos] = float4(1.0);
        irradiance = GetSkyColor(rayDir, true);
        DepthTex[pos] = -1.0;
        break;
      }
      if (pc.MaxBounces == 0) {
        // fake lighting just so that faces are distinguishable
        irradiance *= hit.Normal.x != 0 ? 0.8 : 
          hit.Normal.z != 0 ? 0.5 : 
          hit.Normal.y  < 0 ? 0.3 : 1.0;
      }
      AlbedoTex[pos] = PackAlbedo(albedo, hit.Normal, emission > 0.5);

      float4 projPos = mul(pc.GBuffer->ProjMat, float4(hit.Pos, 1.0));
      float depth = projPos.z / projPos.w;
      DepthTex[pos] = depth;
    } else {
      if (hit.Miss) {
        irradiance += GetSkyColor(rayDir, false) * throughput;
        break;
      }
      throughput *= albedo;
    }

    irradiance += throughput * emission;

    rayDir = normalize(hit.Normal + GetSphereDir(blueNoise.GetSample()));
    rayPos = hit.Pos + hit.Normal * 0.01;
  }

  uint64_t clockEnd = clockARB();
  float clockDiff = (clockEnd - clockStart) / 1000.0;

#if PERF_PRECISE_ITER_HEATMAP
  clockDiff = g_CurrTraversalIters;
#endif

  IrradianceTex[pos] = float4(irradiance, clockDiff);
}

void GetPrimaryRay(int2 screenPos, out float3 rayPos, out float3 rayDir) {
    float2 pos = float2(screenPos) + 0.5;
    if (pc.GBuffer->NumSteadyFrames > 0) {
        float easeWeight = max(1, 10.0 - pc.GBuffer->NumSteadyFrames);
        pos += (MartinR2(pc.GBuffer->NumSteadyFrames) - 0.5) / easeWeight;
    }
    float4 far = mul(pc.GBuffer->InvProjMat, float4(pos, 1, 1));
    rayPos = pc.GBuffer->OriginFrac;
    rayDir = normalize(far.xyz / far.w);
}

//[vk::binding(0, 1)]
//[SamplerDesc(MagFilter.Linear, MinFilter.Linear, MipFilter.Linear, WrapMode.Repeat)]
//public SamplerState g_LinearSampler;

float3 GetSkyColor(float3 dir, bool isPrimary) {
    // return float3(0, 0, 1);
    // For light bounces, sample smaller mips to maybe improve cache coherency.
    return dir;
    //return pc.SkyTexture.Instance.SampleLevel(g_LinearSampler, dir, isPrimary ? 0 : 3) * (isPrimary?1:4);
}
