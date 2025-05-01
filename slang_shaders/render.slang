import core;
import backend_common;
import utilits;
import ray;
import tree64;
import types;

const uint MaxBounces = 0;
const int3 WorldOrigin = int3(0, 0, 0);
const uint NumSteadyFrames = 0;


[vk::push_constant] DispatchParams pc;

[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {

  Ray ray = init_ray(pc.render_data, pos);
  //var blueNoise = BlueNoise(pc.StbnTexture.Instance, pos, pc.GBuffer->FrameNo);

  //float3 irradiance = pc.MaxBounces == 0 ? 1.0 : 0.0;
  //float3 throughput = 1.0;
 
  for (int bounceNo = 0; bounceNo <= MaxBounces; bounceNo++) {
    //PERF_STAT_INC(RayCasts, 1);
    //PERF_TIME_BEGIN(ClocksPerRay);
    HitInfo hit = pc.tree.RayCast(WorldOrigin, ray.pos, ray.dir, bounceNo != 0);
    //PERF_TIME_END(ClocksPerRay);

    float3 albedo;
    float emission;

    if (!hit.Miss) {
      let material = pc.Palette[hit.MaterialId];
      albedo = material.Color;
      emission = material.Emission;
    }

    if (bounceNo == 0) {
      if (hit.Miss) {
        pc.image.Store(pos, float4(1.0));
        //irradiance = GetSkyColor(rayDir, true);
        //pc.GBuffer->DepthTex.Instance[pos] = -1.0;
        break;
      }
      if (pc.MaxBounces == 0) {
        // fake lighting just so that faces are distinguishable
        //irradiance *= hit.Normal.x != 0 ? 0.8 : 
          //hit.Normal.z != 0 ? 0.5 : 
          //hit.Normal.y  < 0 ? 0.3 : 1.0;
      }
      //pc.image.Store(pos, PackAlbedo(albedo, hit.Normal, emission > 0.5));
      pc.image.Store(pos, float4(albedo.x, albedo.y, albedo.z, 1));

      //float4 projPos = mul(pc.GBuffer->ProjMat, float4(hit.Pos, 1.0));
      //float depth = projPos.z / projPos.w;
      //pc.GBuffer->DepthTex.Instance[pos] = depth;
    } else {
      if (hit.Miss) {
        //irradiance += GetSkyColor(rayDir, false) * throughput;
        break;
      }
      //throughput *= albedo;
    }

    //irradiance += throughput * emission;

    ray.dir = normalize(hit.Normal);
    ray.pos = hit.Pos + hit.Normal * 0.01;
  }

  //uint64_t clockEnd = clockARB();
  //float clockDiff = (clockEnd - clockStart) / 1000.0;

  //#if PERF_PRECISE_ITER_HEATMAP
  //clockDiff = g_CurrTraversalIters;
  //#endif

  //pc.GBuffer->IrradianceTex.Instance[pos] = float4(irradiance, clockDiff);
}

