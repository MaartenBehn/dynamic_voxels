public struct DispatchParams {
  RenderData render_data;
  VoxelTree tree;
  RWTexture2D* image;
  Material* Palette;
  uint MaxBounces;
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
  public uint TreeScale;
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
      return f16tof32(Data.x >> 16);
    }
  }
  public property float MetalFuzziness {
    get {
      return (Data.y & 255) / 255.0;
    }
  }
}
