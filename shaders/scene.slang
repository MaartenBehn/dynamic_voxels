import helper_ray;
import tree64;
import debug;

public struct SceneObject {
  public float3 min;
  uint child;
  public float3 max;
  uint exit;

  public property bool leaf { get { return bool(child & 1); } }
  public property uint object_start { get { return child >> 1; } }
  public property uint node_exit { get { return exit; } }
  public property uint leaf_exit { get { return exit; } }
}

public struct DAG64SceneObject {
  public float4x4 mat;
  public float4x4 inv_mat;
  public uint64_t node_alloc;
  public uint64_t data_alloc;
  public uint root_index;
}

public struct Scene {
  uint8_t* start_ptr;
  uint bvh_offset;
  uint bvh_len;

  public __init( uint8_t* start_ptr, uint bvh_offset, uint bvh_len) {
    this.start_ptr = start_ptr;
    this.bvh_offset = bvh_offset;
    this.bvh_len = bvh_len;
  } 

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
        DAG64SceneObject* tree_object = (DAG64SceneObject*)(start_ptr + node.object_start);
        Tree64 tree = Tree64((Node*)(start_ptr + tree_object.node_alloc), start_ptr + tree_object.data_alloc, tree_object.root_index);

        float4x4 mat = tree_object.mat;
        float3 object_ray_pos = mul(float4(ray_pos, 1.0), mat).xyz;
        float3 object_ray_dir = mul(float4(ray_dir, 0.0), mat).xyz;

        HitInfo hit_new = tree.RayCast(object_ray_pos, object_ray_dir, false);
        
        if (!hit_new.Miss && (hit.t >= hit_new.t)) {
          float4x4 inv_mat = tree_object.inv_mat;
          hit_new.Pos = mul(float4(hit_new.Pos, 1.0), inv_mat).xyz; 
          hit_new.Normal = normalize(mul(float4(hit_new.Normal, 0.0), inv_mat).xyz);

          hit = hit_new;
        } else {
          // To debug BVH
          //hit.MaterialId = 1;
        }

        current = node.leaf_exit;
      } else {
        current++;
      }
    }

    return hit;
  }
}




