import tree64;
import helper_math;

public struct Probe {
  public float3 position;
  public uint start_index;
  public uint object_offset;
};

public struct ProbeTree {
  Node* nodes;
  Probe* probes;
  uint start_index;

  public float3 sample_probes(float3 pos) {
    Node node = nodes[start_index];

    uint scale_exp = 21; 
    uint child_index = GetNodeCellIndex(pos, scale_exp);
    uint node_index;
    while ((node.pop_mask >> child_index & 1) != 0 && !node.IsLeaf && scale_exp >= 2) {
      node_index = node.ChildPtr + popcnt_var64(node.pop_mask, child_index);
      node = nodes[node_index];
      scale_exp -= 2;
      child_index = GetNodeCellIndex(pos, scale_exp);
    } 
  }
}
