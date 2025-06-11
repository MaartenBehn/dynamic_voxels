import g_buffer;
import utilits;
import texture_heap;

enum DebugChannel { None, Albedo, Irradiance, Normals, Depth, HeatMap, Variance };

struct DispatchParams {
  GBufferUniforms* GBuffer;
  DebugChannel Channel;
  float2 HeatmapRange;
};
[vk::push_constant] DispatchParams pc;



//[vk::binding(0, 1)]
//[SamplerDesc(MagFilter.Linear, MinFilter.Linear, MipFilter.Linear, WrapMode.Repeat)]
//public SamplerState g_LinearSampler;

[shader("compute")]
[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {
  var AlbedoTex = pc.GBuffer.AlbedoTex.Instance;
  var PrevAlbedoTex = pc.GBuffer.PrevAlbedoTex.Instance;
  var IrradianceTex = pc.GBuffer.IrradianceTex.Instance;
  var PrevIrradianceTex = pc.GBuffer.PrevIrradianceTex.Instance;
  var DepthTex = pc.GBuffer.DepthTex.Instance;
  var PrevDepthTex = pc.GBuffer.PrevDepthTex.Instance;
  var MomentsTex = pc.GBuffer.MomentsTex.Instance;
  var PrevMomentsTex = pc.GBuffer.PrevMomentsTex.Instance;
  var HistoryLenTex = pc.GBuffer.HistoryLenTex.Instance;
  
  float3 albedo = AlbedoTex[pos].rgb;
  float3 irradiance = IrradianceTex[pos].rgb;

  // This is fine as long image was created with USE_SAMPLED
  //float3 albedo = AlbedoTex.SampleLevel(g_LinearSampler, uv, 0).rgb;
  //float3 irradiance = IrradianceTex.SampleLevel(g_LinearSampler, uv, 0).rgb;

  float4 output;
  switch (pc.Channel) {
    default: {            
      float3 color = albedo * irradiance;

      color *= 0.48;
      color = aces_approx(color);
      color = pow(color, 0.45);

      output = float4(color, 1.0);
      break;
    }
    case DebugChannel.Albedo: {
      output = float4(albedo, 1.0);
      break;
    }
    case DebugChannel.Irradiance: {
      output = float4(aces_approx(irradiance), 1.0);
      break;
    }
    case DebugChannel.Normals: {
      float3 normal = UnpackAlbedo(AlbedoTex[pos]).Normal;
      output = float4(normal * 0.5 + 0.5, 1.0);
      break;
    }
    case DebugChannel.Depth: {
      float depth = DepthTex[pos];
      output = float4(float3(depth), 1.0);
      break;
    }
    case DebugChannel.HeatMap: {
      // radiance.w == renderClocks unless overwritten by reproj / SVGF passes.
      float iters = IrradianceTex[pos].w;
      output = float4(viridis_inferno(iters, pc.HeatmapRange), 1);
      break;
    }
    case DebugChannel.Variance: {
      float variance = IrradianceTex[pos].w;
      variance = sqrt(variance) * 3;
      // variance = imageLoad(u_DepthTex, pos).r * 0.1;
      output = float4(variance.xxx, 1.0);
      break;
    }
  }

  pc.GBuffer.output_tex.Instance[pos] = output;
}



