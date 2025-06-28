public struct Random {
    uint _state;

    public __init(uint3 p) {
        uint2 q = 1103515245U * p.xy;
        uint n = q.x ^ (q.y >> 3U);
        _state = n + p.z * 12345u;
    }

    // https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
    [__ref]
    public uint NextU32() {
        uint state = _state;
        _state = _state * 747796405u + 2891336453u;
        uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
        return (word >> 22u) ^ word;
    }
    [__ref]
    public float NextFloat() {
        return float(NextU32()) * (1.0 / 4294967296.0);
    }
};

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

// Returns linear index for 3D grid in XZY order. Sizes **must** be power of two.
public uint GetLinearIndex(uint3 pos, uint sizeXZ, uint sizeY) {
    return (pos.x & (sizeXZ - 1)) +
           (pos.z & (sizeXZ - 1)) * sizeXZ +
           (pos.y & (sizeY - 1)) * (sizeXZ * sizeXZ);
}
// Returns linear index for 3D grid in XZY order. Sizes **must** be constant otherwise this will generate divisions.
public uint3 GetPosFromLinearIndex(uint idx, uint sizeXZ, uint sizeY) {
    uint3 pos;
    pos.x = idx % sizeXZ;
    pos.z = (idx / sizeXZ) % sizeXZ;
    pos.y = (idx / sizeXZ) / sizeXZ;
    return pos;
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

public void InterlockedSetBit(__ref uint64_t dest, uint index, bool value) {
    // Workaround for 64-bit atomics which are slightly fucked on slang's spirv target atm
    uint mask = 1 << (index & 31);
    uint* ptr = (uint*)&dest + (index / 32u);

    if (value) {
        InterlockedOr(*ptr, mask);
    } else {
        InterlockedAnd(*ptr, ~mask);
    }
}
