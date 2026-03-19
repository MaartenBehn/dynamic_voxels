public struct BlueNoise {
    Texture2D<uint8_t2> stbnTexture;
    int3 tilePos;
    uint sampleIdx;

    public __init(Texture2D<uint8_t2> stbn, int2 samplePos, uint frameNo, uint sampleIdx_ = 0) {
        stbnTexture = stbn;
        tilePos = int3(samplePos, frameNo);
        sampleIdx = sampleIdx_ + (frameNo & ~63); // offset by frameNo to hide noise period
    }

    // Returns 2 blue noise samples at the given position and internal increasing sample index.
    // https://developer.nvidia.com/blog/rendering-in-real-time-with-spatiotemporal-blue-noise-textures-part-1/
    [__ref]
    public float2 GetSample() {
        float2 sampleOffset = MartinR2(sampleIdx++);
        int2 pos = (tilePos.xy + int2(sampleOffset * 128)) & 127;
        pos.y += (tilePos.z & 63) * 128;

        return (stbnTexture[pos] + 0.5) / 256.0;
    }
};

// R2 quasirandom sequence
public float2 MartinR2(uint index) {
    return fract(index * float2(0.75487766624669276005, 0.56984029099805326591) + 0.5);
}

public float3 GetSphereDir(float2 sample) {
    float a = sample.x * 6.283185307179586;
    float y = sample.y * 2 - 1;

    float sy = sqrt(1.0 - y * y);
    return float3(sin(a) * sy, y, cos(a) * sy);
}

public bool CheckBounds(uint3 pos, uint sizeXZ, uint sizeY) {
    return (pos.x | pos.z) < sizeXZ && pos.y < sizeY;
}


// Count number of set bits in variable range [0..width]
public uint popcnt_var64(uint64_t mask, uint width) {
    // return popcnt64(mask & ((1ull << mask) - 1));
    uint himask = uint(mask);
    uint count = 0;

    if (width >= 32) {
        count = countbits(himask);
        himask = uint(mask >> 32);
    }
    uint m = 1u << (width & 31u);
    count += countbits(himask & (m - 1u));

    return count;
}
public uint popcnt64(uint64_t mask) {
    uint a = countbits(uint(mask >> 0));
    uint b = countbits(uint(mask >> 32));
    return a + b;
}
public uint tzcnt64(uint64_t mask) {
    uint a = uint(mask >> 0);
    uint b = uint(mask >> 32);
    uint c = firstbitlow(a != 0 ? a : b);
    return a != 0 ? c : c + 32;
}

public float norm_in_range( float x, float2 range ) {
  return (x - range.x) / (range.y - range.x); 
}


// Octahedral Mapping
public float2 dir_to_oct(float3 n) {
    n /= (abs(n.x) + abs(n.y) + abs(n.z));

    float2 uv = n.xy;

    if (n.z < 0.0) {
        uv = (1.0 - abs(uv.yx)) * sign(uv.xy);
    }

    return uv * 0.5 + 0.5;
}

public float3 oct_to_dir(float2 uv) {
    uv = uv * 2.0 - 1.0;

    float3 n = float3(uv.x, uv.y, 1.0 - abs(uv.x) - abs(uv.y));

    if (n.z < 0.0) {
        n.xy = (1.0 - abs(n.yx)) * sign(n.xy);
    }

    return normalize(n);
}

