import texture_heap;         

public struct GBufferUniforms {
  public float4x4 ProjMat, InvProjMat;
  public float4x4 HistoryProjMat, HistoryInvProjMat;

  public float3 position;
  public uint FrameNo;

  public float3 position_frac;
  public uint NumSteadyFrames;

  public float3 HistoryOriginFrac;
  public ImageHandle2D<float4> AlbedoTex;          // rgb: albedo, w: packed normal

  public float3 OriginDelta;
  public ImageHandle2D<float4> PrevAlbedoTex;          // rgb: albedo, w: packed normal

  public ImageHandle2D<float4> IrradianceTex, PrevIrradianceTex;  // rgb: irradiance, w: variance 
  public ImageHandle2D<float> DepthTex, PrevDepthTex;
  public ImageHandle2D<float2> MomentsTex, PrevMomentsTex;
  public ImageHandle2D<uint> HistoryLenTex;
  public ImageHandle2D<float4> output_tex;          
 
  public property uint2 RenderSize {
    [__ref]
    get {
      uint width, height;
      AlbedoTex.Instance.GetDimensions(width, height);
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
