struct Ray {
    float3 pos;
    float3 dir;
    float3 odir; // = 1 / dir
};

Ray init_ray(float3 pos, float3 dir, float2 coord, float2 res) {
    float2 uv = ((coord * 2 - res) / res.y) * float2(-1);

    float3 ro = pos;
    float3 fwd = dir;
    float3 up = float3(0., 0., 1.);
    float3 right = normalize(cross(up, fwd));
    up = cross(fwd, right);
    float3 rd = right * uv.x + up * uv.y + fwd;
    rd = normalize(rd);

    return Ray(ro, rd, float3(1) / rd);
}
