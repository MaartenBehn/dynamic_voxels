import g_buffer;
import texture_heap;
import helper_color;
import helper_math;

enum DebugChannel { None, Albedo, Irradiance, Normals, Depth, HeatMap, Variance, All };

struct DispatchParams {
  GBuffer* GBuffer;
  float2 debug_depth_range;
  float2 debug_heat_map_range;
  DebugChannel Channel;
};
[vk::push_constant] DispatchParams pc;



//[vk::binding(0, 1)]
//[SamplerDesc(MagFilter.Linear, MinFilter.Linear, MipFilter.Linear, WrapMode.Repeat)]
//public SamplerState g_LinearSampler;

[shader("compute")]
[numthreads(8, 8)]
void main(uint2 pos: SV_DispatchThreadID) {
  float4 output;
  switch (pc.Channel) {
    case DebugChannel.None: { output = get_default(pos); break; }
    case DebugChannel.Albedo: { output = get_albedo(pos); break; }
    case DebugChannel.Irradiance: { output = get_irradiance(pos); break; }
    case DebugChannel.Normals: { output = get_normal(pos); break; }
    case DebugChannel.Depth: { output = get_depth(pos); break; }
    case DebugChannel.HeatMap: { output = get_heat(pos); break; }
    case DebugChannel.Variance: { output = get_variance(pos); break; }
    case DebugChannel.All: {
      uint2 screens = uint2(3, 2);
      uint2 large_pos = pos * screens;
      uint2 screen_nr = large_pos / pc.GBuffer.RenderSize; 
      uint2 in_screen = large_pos % pc.GBuffer.RenderSize;

      switch (screen_nr.y) {
        case 0: {
          switch (screen_nr.x) {
            case 0: { output = get_default(in_screen); break; }
            case 1: { output = get_albedo(in_screen); break; }
            case 2: { output = get_irradiance(in_screen); break; }
          }
          break;
        }
        case 1: {
          switch (screen_nr.x) {
            case 0: { output = get_normal(in_screen); break; }
            case 1: { output = get_depth(in_screen); break; }
            case 2: { output = get_variance(in_screen); break; }
          }
          break;
        }
      }
      break;
    }
  }

  pc.GBuffer.output_tex.Instance[pos] = output;
}

float4 get_default(uint2 pos) {
  float3 albedo = pc.GBuffer.AlbedoTex.Instance[pos].rgb;
  float3 irradiance = pc.GBuffer.IrradianceTex.Instance[pos].rgb;

  float3 color = albedo * irradiance;

  color *= 0.48;
  color = aces_approx(color);
  color = pow(color, 0.45);
  return float4(color, 1.0);
}

float4 get_albedo(uint2 pos) {
  return float4(pc.GBuffer.AlbedoTex.Instance[pos].rgb, 1.0);
}

float4 get_irradiance(uint2 pos) {
  float3 irradiance = pc.GBuffer.IrradianceTex.Instance[pos].rgb;
  return float4(aces_approx(irradiance), 1.0);
}

float4 get_normal(uint2 pos) {
  float3 normal = UnpackAlbedo(pc.GBuffer.AlbedoTex.Instance[pos]).Normal;
  return float4(normal * 0.5 + 0.5, 1.0);
}

float4 get_depth(uint2 pos) {
  float depth = pc.GBuffer.DepthTex.Instance[pos];
  return float4(viridis_plasma(norm_in_range(depth, pc.debug_depth_range)), 1.0);
}

float4 get_heat(uint2 pos) {
  int iters = pc.GBuffer.iter_tex.Instance[pos];
  return float4(viridis_inferno(norm_in_range(float(iters), pc.debug_heat_map_range)), 1.0);
}

float4 get_variance(uint2 pos) {
  float variance = pc.GBuffer.IrradianceTex.Instance[pos].w;
  variance = sqrt(variance) * 3;
  return float4(variance.xxx, 1.0);
}







