
// Per-vertex attributes to be assembled from bound vertex buffers.
struct AssembledVertex
{
    float3	position : POSITION;
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
};

[vk::push_constant] RenderData pc;

[shader("vertex")]
VertexStageOutput vertex(
    AssembledVertex assembledVertex)
{
    VertexStageOutput output;

    float3 position = assembledVertex.position;
    float3 color    = assembledVertex.position;

    output.coarseVertex.color = color;
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
