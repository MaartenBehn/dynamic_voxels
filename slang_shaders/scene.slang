import helper_ray;
import tree64;

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
  float4 x_axis;
  float4 y_axis;
  float4 z_axis;
  uint data_start;
  uint root_index;
  uint fill_1;
  uint fill_2;
  public property float4x4 mat { get { return float4x4(x_axis, y_axis, z_axis, float4(0,0,0,1)); } }
}

public struct Scene {
  uint8_t* start_ptr;
  uint bvh_offset;
  uint bvh_len;

  public HitInfo ray_cast(float3 ray_pos, float3 ray_dir, uint bounce_no) {
    SceneObject* bvh = (SceneObject*)(start_ptr + bvh_offset);
    uint current = 0; 
    float3 inv_ray_dir = 1.0 / ray_dir;

    while (current < bvh_len) {
      SceneObject node = bvh[current];

      float t_min, t_max;
      if (!ray_clip_intersect(ray_pos, inv_ray_dir, node.min, node.max, t_min, t_max)) {
        current = node.node_exit;
        continue;
      }

      if (node.leaf) {
        if (node.leaf_nr == 0) {
          Tree64SceneObject* tree_object = (Tree64SceneObject*)(start_ptr + node.object_start);
          Tree64 tree = Tree64((uint8_t*)tree_object, 64, tree_object.data_start, tree_object.root_index); 
           
          HitInfo hit = tree.RayCast(ray_pos, ray_dir, bounce_no != 0);
          hit.MaterialId = 1;
          if (!hit.Miss) {
            return hit;
          }
        }

        current = node.leaf_exit;
      } else {
        current++;
      }
    }

    HitInfo hit;
    hit.MaterialId = 0;
    return hit;
  }

  HitInfo ray_cast_leaf(SceneObject leaf, float3 ray_pos, float3 ray_dir) {
    
    HitInfo hit;
    hit.MaterialId = 0;
    return hit;
  }
}




