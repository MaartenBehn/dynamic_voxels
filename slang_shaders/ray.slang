import types;

struct Ray {
    float3 pos;
    float3 dir;
    float3 odir; // = 1 / dir
};

Ray init_ray(RenderData render_data, float2 coord) {
    float2 uv = ((coord * 2 - render_data.Res) / render_data.Res.y) * float2(-1);

    float3 ro = render_data.Pos;
    float3 fwd = render_data.Dir;
    float3 up = float3(0., 0., 1.);
    float3 right = normalize(cross(up, fwd));
    up = cross(fwd, right);
    float3 rd = right * uv.x + up * uv.y + fwd;
    rd = normalize(rd);

    return Ray(ro, rd, float3(1) / rd);
}
