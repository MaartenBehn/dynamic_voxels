import ray_manager;
import scene;
import helper_ray;
import texture_heap;

public struct DispatchParams {
  RayManager rm;
  Scene scene;
};

[vk::push_constant] DispatchParams pc;

// Based on https://github.com/dubiousconst282/VoxelRT
[shader("compute")]
[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {
  RayRunner rr = RayRunner(pos, pc.rm);
  bool hit = pc.scene.ray_cast(rr.ray_pos, rr.ray_dir);
  pc.rm.g_buffer.DepthTex.Instance[pos] = hit ? 1.0 : 0.0;
 
  /*
  for (int bounce_no = 0; bounce_no <= pc.rm.max_bounces; bounce_no++) {
    rr.prepare_trace();

    HitInfo hit = pc.scene.ray_cast(rr.ray_pos, rr.ray_dir);
    
    rr.after_trace(pos, hit, pc.rm, bounce_no);
  }

  rr.finish(pos, pc.rm);
  */
}



