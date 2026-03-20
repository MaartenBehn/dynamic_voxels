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
  public uint size;
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

  public ObjectHitInfo ray_cast(float3 ray_pos, float3 ray_dir) {
    SceneObject* bvh = (SceneObject*)(start_ptr + bvh_offset);
    uint current = 0; 
    float3 inv_ray_dir = 1.0 / ray_dir;

    ObjectHitInfo object_hit;
    while (current < bvh_len) {
      SceneObject node = bvh[current];

      float t_min, t_max;
      if (!ray_clip_intersect(ray_pos, inv_ray_dir, node.min, node.max, t_min, t_max)) {
        current = node.node_exit;
        continue;
      }

      if (node.leaf) {
        DAG64SceneObject* object = (DAG64SceneObject*)(start_ptr + node.object_start);
        Tree64 tree = Tree64((Node*)(start_ptr + object.node_alloc), start_ptr + object.data_alloc, object.root_index);

        float4x4 mat = object.mat;
        float3 object_ray_pos = mul(float4(ray_pos, 1.0), object.mat).xyz;
        float3 object_ray_dir = mul(float4(ray_dir, 0.0), object.mat).xyz;

        HitInfo hit = tree.RayCast(object_ray_pos, object_ray_dir, false);

        
        if (hit.hit && (object_hit.t >= hit.t)) {
          object_hit = ObjectHitInfo(hit, object, current, object_ray_dir);
        } else {
          // To debug BVH
          //object_hit.MaterialId = 1;
        }

        current = node.leaf_exit;
      } else {
        current++;
      }
    }

    return object_hit;
  }

  public ObjectHitInfo ray_cast_from(HitObject last_hit) {
    Tree64 tree = Tree64(
      (Node*)(start_ptr + last_hit.object.node_alloc), 
      start_ptr + last_hit.object.data_alloc, 
      last_hit.object.root_index);

    HitInfo hit = tree.RayCast(last_hit.pos, last_hit.ray_dir, false);

    ObjectHitInfo object_hit;
    if (hit.hit) {
      object_hit = ObjectHitInfo(hit, last_hit.object, last_hit.bvh_index, last_hit.ray_dir);
    }
    
    float3 ray_pos = mul(float4(last_hit.pos, 1.0), last_hit.object.inv_mat).xyz; 
    float3 ray_dir = mul(float4(last_hit.ray_dir, 0.0), last_hit.object.inv_mat).xyz; 
    
    SceneObject* bvh = (SceneObject*)(start_ptr + bvh_offset);
    uint current = 0; 
    float3 inv_ray_dir = 1.0 / ray_dir;

    while (current < bvh_len) {
      SceneObject node = bvh[current];

      float t_min, t_max;
      if (current == last_hit.bvh_index || !ray_clip_intersect(ray_pos, inv_ray_dir, node.min, node.max, t_min, t_max)) {
        current = node.node_exit;
        continue;
      }

      if (node.leaf) {
        DAG64SceneObject* object = (DAG64SceneObject*)(start_ptr + node.object_start);
        Tree64 tree = Tree64((Node*)(start_ptr + object.node_alloc), start_ptr + object.data_alloc, object.root_index);

        float4x4 mat = object.mat;
        float3 object_ray_pos = mul(float4(ray_pos, 1.0), object.mat).xyz;
        float3 object_ray_dir = mul(float4(ray_dir, 0.0), object.mat).xyz;

        HitInfo hit = tree.RayCast(object_ray_pos, object_ray_dir, false);
        
        if (hit.hit && (object_hit.t >= hit.t)) {
          object_hit = ObjectHitInfo(hit, object, current, object_ray_dir);
        } else {
          // To debug BVH
          //object_hit.MaterialId = 1;
        }

        current = node.leaf_exit;
      } else {
        current++;
      }
    }

    return object_hit;    
  }
}

public struct HitObject {
  public DAG64SceneObject* object;
  public uint bvh_index;
  public float3 ray_dir;
  public float3 pos;
}

public struct ObjectHitInfo {
  public HitObject object;
  public float3 normal;
  public uint material_id;
  public float t;

  public __init() {
    t = float::maxValue;
    material_id = 0;
    normal = 0;
    object.object  = (DAG64SceneObject*)0;
    object.ray_dir = 0;
    object.bvh_index = 0;
    object.pos = 0;
  }

  public __init(HitInfo hit, DAG64SceneObject* object, uint bvh_index, float3 ray_dir) {
    this.object = HitObject(object, bvh_index, ray_dir, hit.Pos);
    t = hit.t;
    normal = hit.Normal;
    material_id = hit.MaterialId;
  }

  public property bool miss { get { return material_id == 0; } }
  public property bool hit { get { return material_id != 0; } }

  public float3 global_pos() {
    return mul(float4(object.pos, 1.0), object.object.inv_mat).xyz;
  }

  public float3 global_normal() {
    return normalize(mul(float4(normal, 0.0), object.object.inv_mat).xyz);
  }
}




