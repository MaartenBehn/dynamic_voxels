import g_buffer;
import helper_math;

public interface IRayCastable {
  public HitInfo RayCast(int3 worldOrigin, float3 origin, float3 dir, bool coarse);
}

public struct HitInfo {
  public float t;
  public float3 Pos;
  public float3 Normal;
  public uint MaterialId;

  public __init() {
    t = float::maxValue;
    Pos = 0;
    Normal = 0;
    MaterialId = 0;
  }

  public property bool Miss {
    get { return MaterialId == 0; }
  }
}

public void GetPrimaryRay(int2 screenPos, out float3 rayPos, out float3 rayDir, GBuffer* g_buffer) {
    float2 pos = float2(screenPos) + 0.5;
    if (g_buffer.NumSteadyFrames > 0) {
        float easeWeight = max(1, 10.0 - g_buffer.NumSteadyFrames);
        pos += (MartinR2(g_buffer.NumSteadyFrames) - 0.5) / easeWeight;
    }
    float4 far = mul(g_buffer.InvProjMat, float4(pos, 1, 1));
    rayPos = g_buffer.position;
    rayDir = normalize(far.xyz / far.w);
}

public bool ray_clip_intersect(float3 origin, float3 inv_dir, float3 min_pos, float3 max_pos, out float t_min, out float t_max) {
  float3 is_positive = float3(float(inv_dir.x > 0), float(inv_dir.y > 0), float(inv_dir.z >= 0));     
  float3 is_negative = 1.0f - is_positive;

  float3 left_side = is_positive * min_pos + is_negative * max_pos;
  float3 right_side = is_positive * max_pos + is_negative * min_pos;

  float3 left_side_times_one_over_dir = (left_side - origin) * inv_dir;
  float3 right_side_times_one_over_dir = (right_side - origin) * inv_dir;

  t_min = max(left_side_times_one_over_dir.x, max(left_side_times_one_over_dir.y, left_side_times_one_over_dir.z));
  t_max = min(right_side_times_one_over_dir.x, min(right_side_times_one_over_dir.y, right_side_times_one_over_dir.z));
 
  return t_max > 0 && t_max > t_min;
}


public static const int kDefaultOpaqueMaterialId = 254;

public struct Material {
    public uint2 Data;

    public property float3 Color {
        get {
            uint3 mask = uint3(31, 63, 31);
            float3 color = float3(Data.xxx >> uint3(11, 5, 0) & mask) / float3(mask);
            return color * color;  // srgb gamma hack
        }
    }
   
    public property float Emission {
        get {
            return float(Data.x >> 16);
        }
    }
  
    public property float MetalFuzziness {
        get {
            return (Data.y & 255) / 255.0;
        }
    }
}

