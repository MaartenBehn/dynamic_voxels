layout(binding = 0, rgba8) public WTexture2D img;

public struct DispatchParams {
  public RenderData render_data;
  public VoxelTree tree;
  public float4* Palette;
  public uint MaxBounces;
};

public struct RenderData {
  float PackedData[8];

  public property float3 Pos {
  get { return float3(PackedData[0], PackedData[1], PackedData[2]);  }
  }
  public property float3 Dir {
  get { return float3(PackedData[3], PackedData[4], PackedData[5]); }
  }
  public property float2 Res {
  get { return float2(PackedData[6], PackedData[7]); }
  }
}

public struct VoxelTree {
  public int3 worldOrigin;
  public uint start_index;
  public Node* TreeNodes;
  public uint8_t* LeafData;
}

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
