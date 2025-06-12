import ray_manager;
import tree64;
import helper_ray;

public struct DispatchParams {
  RayManager rm;
  Tree64 tree;
};

[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {
  RayRunner rr = RayRunner(pos, pc.rm); 
  
  for (int bounce_no = 0; bounce_no <= pc.rm.max_bounces; bounce_no++) {
    rr.prepare_trace();

    HitInfo hit = pc.tree.RayCast(int3(pc.rm.g_buffer.position), rr.ray_pos, rr.ray_dir, bounce_no != 0);
    
    rr.after_trace(pos, hit, pc.rm, bounce_no);
  }

  rr.finish(pos, pc.rm);
}



