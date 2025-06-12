import g_buffer;
import helper_math;

public interface IRayCastable {
  public HitInfo RayCast(int3 worldOrigin, float3 origin, float3 dir, bool coarse);
}

public struct HitInfo {
  public float Dist;
  public float3 Pos;
  public float3 Normal;
  public float2 FaceUV;
  public uint MaterialId;

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
    rayPos = g_buffer.position_frac;
    rayDir = normalize(far.xyz / far.w);
}

public float3 ClipRayToAABB(float3 origin, float3 dir, float3 bbMin, float3 bbMax) {
  float3 invDir = 1.0 / dir;
  float3 t1 = (bbMin - origin) * invDir;
  float3 t2 = (bbMax - origin) * invDir;
  float3 temp = t1;
  t1 = min(temp, t2);
  t2 = max(temp, t2);

  float tmin = max(t1.x, max(t1.y, t1.z));
  float tmax = min(t2.x, min(t2.y, t2.z));

  return tmin > 0 && tmin < tmax ? origin + dir * tmin : origin;
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

