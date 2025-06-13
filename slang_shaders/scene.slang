import helper_ray;

public struct SceneObject {
  public float3 min;
  uint child;
  public float3 max;
  uint exit;

  public property bool leaf { get { return bool(child & 1); } }
  public property uint leaf_nr { get { return exit & 1; } }
  public property uint child_nodes { get { return child >> 1; } }
  public property uint object_start { get { return child >> 1; } }
  public property uint node_exit { get { return exit; } }
  public property uint leaf_exit { get { return exit >> 1; } }
}

public struct Tree64SceneObject {
  float4x4 mat; 
  uint start_index;
}

public struct Scene {
  uint8_t* start_ptr;
  uint bvh_offset;
  uint root_object_index;
  uint bvh_len;

  public bool ray_cast(float3 origin, float3 ray_dir) {
    SceneObject* bvh = start_ptr + bvh_offset;
    uint current = root_object_index; 
    float3 inv_ray_dir = 1.0 / ray_dir;

    while (current < bvh_len) {
      SceneObject node = bvh[current];

      float t_min, t_max;
      if (!ray_aabb_intersect(ray_pos, inv_ray_dir, node.min, node.max, t_min, t_max)) {
        current = node.node_exit;
        continue;
      }

      if (node.leaf) {
        return true;       

        current = node.leaf_exit;
      } else {
        current++;
      }
    }
  } 
}




