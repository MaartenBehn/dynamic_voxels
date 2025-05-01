
#extension GL_EXT_scalar_block_layout : require

layout(binding = 0, rgba8) uniform writeonly Texture2D img;

layout(binding = 1, std430) struct {
    float4 pos; // w is screen_size_x
    float4 dir; // w is screen_size_y
    float4 data; // time in sec
} render_buffer;

#define POS render_buffer.pos.xyz
#define DIR render_buffer.dir.xyz
#define RES_X render_buffer.pos.w
#define RES_Y render_buffer.dir.w
#define RES vec2(RES_X, RES_Y)
#define TIME render_buffer.data.x;

#define MAX_CGS_TREE_SIZE 2048



public struct Node {
  uint PackedData[3];

  public property bool IsLeaf {
  get { return (PackedData[0] & 1) != 0; }
  }
  public property uint ChildPtr {
  get { return PackedData[0] >> 1; }
  }
  public property uint64_t PopMask {
  get { return PackedData[1] | uint64_t(PackedData[2]) << 32; }
  }
}

layout(binding = 2) public RWStructuredBuffer<Node> TreeNodes;

#define MAX_COLOR_BUFFER_SIZE 256
layout(binding = 3) uniform struct {
    float4[MAX_COLOR_BUFFER_SIZE] data;
} color_buffer;

#define COLOR_BUFFER color_buffer.data;

