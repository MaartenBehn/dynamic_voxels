import ray_manager;
import scene;
import helper_ray;
import texture_heap;
import debug;

public struct DispatchParams {
  RayManager rm;
  Scene scene;
  bool debug;
};

[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {
  if (pos.x == 500 && pos.y == 500) {
    DEBUG_PRINT = pc.debug;
  }

  RayRunner rr = RayRunner(pos, pc.rm);
 
  for (int bounce_no = 0; bounce_no <= pc.rm.max_bounces; bounce_no++) {
    rr.prepare_trace();

    HitInfo hit = pc.scene.ray_cast(rr.ray_pos, rr.ray_dir, bounce_no);
    
    rr.after_trace(pos, hit, pc.rm, bounce_no);
  }

  rr.finish(pos, pc.rm);
}



