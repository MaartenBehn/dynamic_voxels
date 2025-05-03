import core;
import backend_common;
import utilits;
import ray;
import tree64;
import types;

static uint MaxBounces = 1;
static int3 WorldOrigin = int3(0, 0, 0);
static uint NumSteadyFrames = 0;


[vk::push_constant] DispatchParams pc;

[numthreads(8, 8)]
void compute_main(uint2 pos: SV_DispatchThreadID) {
  if (pos.x >= pc.render_data.Res.x || pos.y >= pc.render_data.Res.y ) {
    return;
  } 
  
  Ray ray = init_ray(pc.render_data, pos);

  float4 color = float4(ray.dir, 1.);
  
  Interval interval = init_interval();
  if (ray_aabb_intersect(ray, float3(0), float3(10), interval)) {
    ray.pos = get_ray_pos(ray, interval.t_min + 0.1); 

    for (int bounceNo = 0; bounceNo <= MaxBounces; bounceNo++) {
      HitInfo hit = pc.tree.RayCast(ray.pos, ray.dir, bounceNo != 0);

      if (!hit.Miss) {
        color = pc.Palette[hit.MaterialId];
      }
    
      ray.dir = normalize(hit.Normal);
      ray.pos = hit.Pos + hit.Normal * 0.01;
    }
  }
 
  
  img.Store(pos, color);
}

