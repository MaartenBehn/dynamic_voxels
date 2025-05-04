import texture_heap;
import g_buffer;
import utilits;

enum DebugChannel { None, Albedo, Irradiance, Normals, HeatMap, Variance };

struct DispatchParams {
  GBufferUniforms* GBuffer;
  ImageHandle2D<float4> output;
  DebugChannel Channel;
  float2 HeatmapRange;
};
[vk::push_constant] DispatchParams pc;

[shader("compute")]
[numthreads(8, 8)]
void compute_main(float2 uv: TEXCOORD0) {
  int2 pos = int2(uv * pc.GBuffer->RenderSize);
  // float3 albedo = pc.GBuffer->AlbedoTex.Instance[pos].rgb;
  // float3 irradiance = pc.GBuffer.IrradianceTex.Instance[pos].rgb;

  // This is fine as long image was created with USE_SAMPLED
  float3 albedo = reinterpret<TextureHandle2D>(pc.GBuffer->AlbedoTex).Instance.SampleLevel(g_LinearSampler, uv, 0).rgb;
  float3 irradiance = reinterpret<TextureHandle2D>(pc.GBuffer->IrradianceTex).Instance.SampleLevel(g_LinearSampler, uv, 0).rgb;

  float4 output;
  switch (pc.Channel) {
    default: {            
      float3 color = albedo * irradiance;

      color *= 0.48;
      color = aces_approx(color);
      color = pow(color, 0.45);

      output = float4(color, 1.0);
    }
    case DebugChannel.Albedo: {
      output = float4(albedo, 1.0);
    }
    case DebugChannel.Irradiance: {
      output = float4(aces_approx(irradiance), 1.0);
    }
    case DebugChannel.Normals: {
      float3 normal = UnpackAlbedo(pc.GBuffer.AlbedoTex.Instance[pos]).Normal;
      output = float4(normal * 0.5 + 0.5, 1.0);
    }
    case DebugChannel.HeatMap: {
      // radiance.w == renderClocks unless overwritten by reproj / SVGF passes.
      float iters = pc.GBuffer.IrradianceTex.Instance[pos].w;
      output = float4(viridis_inferno(iters, pc.HeatmapRange), 1);
    }
    case DebugChannel.Variance: {
      float variance = pc.GBuffer.IrradianceTex.Instance[pos].w;
      variance = sqrt(variance) * 3;
      // variance = imageLoad(u_DepthTex, pos).r * 0.1;
      output = float4(variance.xxx, 1.0);
    }
  }

  pc.output.Instance[pos] = output;
}



