[vk::binding(0, 0)] public RWTexture2D<float4> AlbedoTex;               // rgb: albedo, w: packed normal
[vk::binding(1, 0)] public RWTexture2D<float4> PrevAlbedoTex;          
[vk::binding(2, 0)] public RWTexture2D<float4> IrradianceTex;           // rgb: irradiance, w: variance 
[vk::binding(3, 0)] public RWTexture2D<float4> PrevIrradianceTex;           
[vk::binding(4, 0)] public RWTexture2D<float> DepthTex;           
[vk::binding(5, 0)] public RWTexture2D<float> PrevDepthTex;           
[vk::binding(6, 0)] public RWTexture2D<float2> MomentsTex;           
[vk::binding(7, 0)] public RWTexture2D<float2> PrevMomentsTex;           
[vk::binding(8, 0)] public RWTexture2D<uint> HistoryLenTex;           
//[vk::binding(9, 0)] public RWTexture2D<float4> Output;           

public struct GBufferUniforms {
  public float4x4 ProjMat, InvProjMat;
  public float4x4 HistoryProjMat, HistoryInvProjMat;

  public float3 position;
  public uint FrameNo;

  public float3 position_frac;
  public uint NumSteadyFrames;

  public float3 HistoryOriginFrac;
  uint fill_1;

  public float3 OriginDelta;
  uint fill_2;
 
  public property uint2 RenderSize {
    [__ref]
    get {
      uint width, height;
      AlbedoTex.GetDimensions(width, height);
      return uint2(width, height);
    }
  }

  [__ref]
  public bool CheckBounds(int2 pos) {
    return all(uint2(pos) < RenderSize);
  }
};

public struct AlbedoData {
    public float3 Color;
    public float3 Normal;
    public bool Emissive;
};
public static AlbedoData UnpackAlbedo(float4 value) {
    AlbedoData data;
    data.Color = value.rgb;
    
    uint wbits = uint(value.w * 255 + 0.5);
    data.Normal = float3(wbits >> uint3(0, 2, 4) & 3u) - 1;
    data.Emissive = (wbits >> 6 & 1) != 0;
    return data;
}
public static float4 PackAlbedo(float3 color, float3 normal, bool emiss) {
    uint3 inorm = uint3(clamp(normal + 1, 0, 3));
    float w = float(inorm.x << 0 | inorm.y << 2 | inorm.z << 4 | (emiss ? 1 << 6 : 0)) / 255.0;
    return float4(color, w);
}
