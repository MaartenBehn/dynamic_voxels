import helper_ray;

// Per-vertex attributes to be assembled from bound vertex buffers.
struct AssembledVertex
{
  float3	position : POSITION;
  uint    material_id;
  float3	normal;
};

// Output of the vertex shader, and input to the fragment shader.
struct CoarseVertex
{
  float3 color;
};

// Output of the fragment shader
struct Fragment
{
  float4 color;
};

struct VertexStageOutput
{
  CoarseVertex    coarseVertex    : CoarseVertex;
  float4          sv_position     : SV_Position;
};

public struct RenderData {
  public float4x4 proj_mat;
  public Material* palette;
};

[vk::push_constant] RenderData pc;

  [shader("vertex")]
VertexStageOutput vertex(
    AssembledVertex assembledVertex)
{
  VertexStageOutput output;

  float3 position = assembledVertex.position.xyz;
  float3 normal = assembledVertex.normal;
  float3 color = pc.palette[assembledVertex.material_id].Color;

  float3 sun_dir = normalize(float3(-0.3, 0.7, 0.6));

  float light = dot(normal, sun_dir) * 0.5 + 0.5;
  light = pow(light, 1.2);

  // Sun color
  //float3 sunColor = float3(1.0, 0.95, 0.8);

  // Final shaded color
  float3 litColor = color * light; 

  output.coarseVertex.color = litColor;
  //output.coarseVertex.color = assembledVertex.normal;
  output.sv_position = mul(pc.proj_mat, float4(position, 1.0));

  return output;
}

// Fragment Shader

[shader("fragment")]
Fragment fragment(
    CoarseVertex coarseVertex : CoarseVertex) : SV_Target
{
  float3 color = coarseVertex.color;

  Fragment output;
  output.color = float4(color, 1.0);
  return output;
}
