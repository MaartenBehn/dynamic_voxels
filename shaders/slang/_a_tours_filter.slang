import g_buffer;
import texture_heap;
import helper_color;

// SVGF-based denoising filter
// - https://research.nvidia.com/publication/2017-07_spatiotemporal-variance-guided-filtering-real-time-reconstruction-path-traced
// - https://alain.xyz/blog/ray-tracing-denoising
// - https://diharaw.github.io/post/adventures_in_hybrid_rendering/

struct DispatchParams {
  GBuffer* GBuffer;
  ImageHandle2D<float4> InIrradianceTex, OutIrradianceTex;  // rgb: irradiance, w: variance 
  uint passIndex;
};
[vk::push_constant] DispatchParams pc;

[shader("compute")]
[numthreads(8, 8, 1)]
void compute_main(int2 pos: SV_DispatchThreadID) {
  float centerDepth = pc.GBuffer.DepthTex.Instance[pos].r;
  float4 centerIrradiance = pc.InIrradianceTex.Instance[pos];

  if (centerDepth < 0) {
    pc.OutIrradianceTex.Instance[pos] = centerIrradiance;
    return;
  }

  float centerVariance = getFilteredVariance(pos);
  // TODO Unpack normal func
  float3 centerNormal = UnpackAlbedo(pc.GBuffer.AlbedoTex.Instance[pos]).Normal;
  float centerLuma = get_luminance(centerIrradiance.rgb);

  float lumaPhi = sqrt(max(0.0001, centerVariance)) * 4.0;
  float depthPhi = 0.001;

  float4 sumIrradiance = centerIrradiance;
  float wsum = 1.0;

  const int r = 2;
  const float kernel[] = { 3.0 / 8, 1.0 / 4, 1.0 / 16 }; // wtf is it? Falcor uses different kernel from paper

  for (int ky = -r; ky <= r; ky++) {
    for (int kx = -r; kx <= r; kx++) {
      if (kx == 0 && ky == 0) continue;

      int2 samplePos = pos + (int2(kx, ky) << pc.passIndex);
      if (!pc.GBuffer.CheckBounds(samplePos)) continue;

      float4 irradiance = pc.InIrradianceTex.Instance[samplePos];
      float w_luma = abs(get_luminance(irradiance.rgb) - centerLuma) / lumaPhi;

      float3 normal = UnpackAlbedo(pc.GBuffer.AlbedoTex.Instance[samplePos]).Normal;
      float w_normal = pow(clamp(dot(normal, centerNormal), 0.001, 1.0), 128);

      float depth = pc.GBuffer.DepthTex.Instance[samplePos].r;
      float w_depth = abs(centerDepth - depth) / (length(int2(kx, ky) << pc.passIndex) + 0.001);
      // FIXME: this won't do anything, depth should be linear

      float w = kernel[abs(kx)] * kernel[abs(ky)];
      w *= exp(-(w_luma + w_depth)) * w_normal;

      sumIrradiance += float4(irradiance.rgb * w, irradiance.w * (w * w));
      wsum += w;
    }
  }
  if (wsum < 0.001) wsum = 0.001;
  sumIrradiance.rgb /= wsum;
  sumIrradiance.w /= (wsum * wsum);
  pc.OutIrradianceTex.Instance[pos] = sumIrradiance;
}

// From https://github.com/NVIDIAGameWorks/Falcor/blob/master/Source/RenderPasses/SVGFPass/SVGFAtrous.ps.slang
// computes a 3x3 gaussian blur of the variance, centered around
// the current pixel
float getFilteredVariance(int2 pos) {
  const float kernel[2][2] = {
    { 1.0 / 4.0, 1.0 / 8.0 },
    { 1.0 / 8.0, 1.0 / 16.0 },
  };
  float sum = 0.0;

  for (int ky = -1; ky <= 1; ky++) {
    for (int kx = -1; kx <= 1; kx++) {
      sum += pc.InIrradianceTex.Instance[pos + int2(kx, ky)].w * kernel[abs(kx)][abs(ky)];
    }
  }
  return sum;
}



