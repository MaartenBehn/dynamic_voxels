import core;
import backend_common;
import utilits;
import ray;
import tree64;
import binding;

const int MaxBounces = 0;
const float3 WorldOrigin = float3(0, 0, 0);
const uint NumSteadyFrames = 0;

[numthreads(8, 8)]
void ComputeMain(uint2 pos: SV_DispatchThreadID) {
    
    Ray ray = init_ray(POS, DIR, pos, RES);
    //var blueNoise = BlueNoise(pc.StbnTexture.Instance, pos, pc.GBuffer->FrameNo);

    //float3 irradiance = pc.MaxBounces == 0 ? 1.0 : 0.0;
    //float3 throughput = 1.0;

    for (int bounceNo = 0; bounceNo <= MaxBounces; bounceNo++) {
        //PERF_STAT_INC(RayCasts, 1);
        //PERF_TIME_BEGIN(ClocksPerRay);
        HitInfo hit = pc.Map.RayCast(WorldOrigin, rayPos, rayDir, bounceNo != 0);
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
                pc.GBuffer->AlbedoTex.Instance[pos] = float4(1.0);
                irradiance = GetSkyColor(rayDir, true);
                pc.GBuffer->DepthTex.Instance[pos] = -1.0;
                break;
            }
            if (pc.MaxBounces == 0) {
                // fake lighting just so that faces are distinguishable
                irradiance *= hit.Normal.x != 0 ? 0.8 : 
                              hit.Normal.z != 0 ? 0.5 : 
                              hit.Normal.y  < 0 ? 0.3 : 1.0;
            }
            pc.GBuffer->AlbedoTex.Instance[pos] = PackAlbedo(albedo, hit.Normal, emission > 0.5);

            float4 projPos = mul(pc.GBuffer->ProjMat, float4(hit.Pos, 1.0));
            float depth = projPos.z / projPos.w;
            pc.GBuffer->DepthTex.Instance[pos] = depth;
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

    //#if PERF_PRECISE_ITER_HEATMAP
    //clockDiff = g_CurrTraversalIters;
    //#endif
    
    pc.GBuffer->IrradianceTex.Instance[pos] = float4(irradiance, clockDiff);
}

