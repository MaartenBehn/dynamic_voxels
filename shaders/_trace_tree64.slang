import ray_manager;
import tree64;
import helper_ray;

public struct DispatchParams {
  float4x4 mat;
  float4x4 inv_mat;
  RayManager rm;
  Tree64 tree;
};

[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void main(uint2 pos: SV_DispatchThreadID) {
  RayRunner rr = RayRunner(pos, pc.rm); 
  
  for (int bounce_no = 0; bounce_no <= pc.rm.max_bounces; bounce_no++) {
    rr.prepare_trace();

    float3 object_ray_pos = mul(float4(rr.ray_pos, 1.0), pc.mat).xyz;
    float3 object_ray_dir = mul(float4(rr.ray_dir, 0.0), pc.mat).xyz;

    HitInfo hit = pc.tree.RayCast(object_ray_pos, object_ray_dir, bounce_no != 0);

    hit.Pos = mul(float4(hit.Pos, 1.0), pc.inv_mat).xyz; 
    hit.Normal = normalize(mul(float4(hit.Normal, 0.0), pc.inv_mat).xyz);
        
    rr.after_trace(pos, hit, pc.rm, bounce_no);
  }

  rr.finish(pos, pc.rm);
}



