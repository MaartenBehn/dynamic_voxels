import helper_ray;
import tree64;
import debug;

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
  float3 x_axis;
  uint data_start;
  float3 y_axis;
  uint root_index;
  float3 z_axis;
  uint fill_1;
  float3 w_axis;
  uint fill_2;
  float4x4 inv_mat;

  property float4x4 mat { get {  
    return float4x4(float4(x_axis, 0), float4(y_axis, 0), float4(z_axis, 0), float4(w_axis, 1));
  }}
}

public struct Scene {
  uint8_t* start_ptr;
  uint bvh_offset;
  uint bvh_len;

  public HitInfo ray_cast(float3 ray_pos, float3 ray_dir, uint bounce_no) {
    SceneObject* bvh = (SceneObject*)(start_ptr + bvh_offset);
    uint current = 0; 
    float3 inv_ray_dir = 1.0 / ray_dir;

    HitInfo hit;
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
          Tree64 tree = Tree64((uint8_t*)tree_object, 128, tree_object.data_start, tree_object.root_index);
          
          float4x4 mat = tree_object.mat;
          float3 object_ray_pos = mul(float4(ray_pos, 1.0), mat).xyz;
          float3 object_ray_dir = mul(float4(ray_dir, 0.0), mat).xyz;
           
          HitInfo hit_new = tree.RayCast(object_ray_pos, object_ray_dir, bounce_no != 0);

          float4x4 inv_mat = tree_object.inv_mat;
          hit_new.Pos = mul(float4(hit_new.Pos, 1.0), inv_mat).xyz; 
          hit_new.Normal = mul(float4(hit_new.Normal, 0.0), inv_mat).xyz;

          if (!hit_new.Miss && (hit.t >= hit_new.t)) {
            hit = hit_new;
          }
        }

        current = node.leaf_exit;
      } else {
        current++;
      }
    }

    return hit;
  }
}




