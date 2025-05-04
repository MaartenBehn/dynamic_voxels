
[vk::binding(0, 0)] __DynamicResource TextureHeap[];
[vk::binding(1, 0)] __DynamicResource RWTextureHeap[];

[vk::binding(0, 1)]
[SamplerDesc(MagFilter.Linear, MinFilter.Linear, MipFilter.Linear, WrapMode.Repeat)]
public SamplerState g_LinearSampler;

public struct ImageHandle<T> {
    uint DescriptorIndex;
}

__generic<T: ITexelElement, Shape: __ITextureShape, let isArray: int, let isMS: int, let sampleCount: int, let access: int, let isShadow: int, let isCombined: int, let format: int>
public extension ImageHandle<_Texture<T, Shape, isArray, isMS, sampleCount, access, isShadow, isCombined, format>> {
    typealias TextureType = _Texture<T, Shape, isArray, isMS, sampleCount, access, isShadow, isCombined, format>;

    public property TextureType Instance {
        get { return (access != 0 ? RWTextureHeap : TextureHeap)[DescriptorIndex].as<TextureType>(); }
    }
}

public typealias TextureHandle2D<T: ITexelElement = float4> = ImageHandle<Texture2D<T>>;
public typealias TextureHandle3D<T: ITexelElement = float4> = ImageHandle<Texture3D<T>>;
public typealias ImageHandle2D<T: ITexelElement = float4> = ImageHandle<RWTexture2D<T>>;
public typealias ImageHandle3D<T: ITexelElement = float4> = ImageHandle<RWTexture3D<T>>;

public typealias RWTextureCube<T: ITexelElement = float4> = _Texture<T, __ShapeCube, 0, 0, 0, 1, 0, 0, 0>;

public typealias TextureHandleCube<T: ITexelElement = float4> = ImageHandle<TextureCube<T>>;
public typealias ImageHandleCube<T: ITexelElement = float4> = ImageHandle<RWTextureCube<T>>;

public typealias ImageHandleArray2D<T: ITexelElement = float4> = ImageHandle<RWTexture2DArray<T>>;

#define DESTRUCT_VEC1(var) var.x
#define DESTRUCT_VEC2(var) var.x, var.y
#define DESTRUCT_VEC3(var) var.x, var.y, var.z

#define DEF_TEXTURE_SIZE_EXT(shape, isArray, dim) \
    __generic<T: ITexelElement, let access : int, let isShadow : int, let isCombined : int, let format : int> \
    public extension _Texture<T, shape, isArray, 0, 0, access, isShadow, isCombined, format> { \
        public property uint##dim Size { \
            get { \
                uint##dim res; \
                GetDimensions(DESTRUCT_VEC##dim(res)); \
                return res; \
            } \
        } \
    }

DEF_TEXTURE_SIZE_EXT(__Shape2D, 0, 2);
DEF_TEXTURE_SIZE_EXT(__Shape2D, 1, 3);
DEF_TEXTURE_SIZE_EXT(__Shape3D, 0, 3);

DEF_TEXTURE_SIZE_EXT(__ShapeCube, 0, 2);

/*
__generic<T>
__prefix T operator *(TextureHandle<T> value) {
    return value.NonUniformInstance;
}
void Foo(TextureRef2D tex) {
    float4 d = (*tex).Load(int3(0));
}*/

// These would ideally be enums but slang reflection doesn't support getting values yet
public class MagFilter { public static const int Nearest = 0, Linear = 1; };
public class MinFilter { public static const int Nearest = 0, Linear = 1; };
public class MipFilter { public static const int Nearest = 0, Linear = 1, Anisotropic = 2; };

public class WrapMode { public static const int Repeat = 0, ClampToEdge = 1, MirroredRepeat = 2, MirrorClampToEdge = 3; };
public class CompareOp { public static const int None = 0, Eq = 1, Ne = 2, Gt = 3, Lt = 4, Ge = 5, Le = 6; };

// Embeds an immutable sampler to the binding where this attribute is applied.
[__AttributeUsage(_AttributeTargets.Var)]
public struct SamplerDescAttribute {
    int magFilter = MagFilter.Linear;
    int minFilter = MinFilter.Linear;
    int mipFilter = MipFilter.Linear;
    int wrap = WrapMode.Repeat;
};

// Not implemented
[__AttributeUsage(_AttributeTargets.Var)]
public struct ShadowSamplerDescAttribute {
    int magFilter = MagFilter.Linear;
    int minFilter = MinFilter.Linear;
    int mipFilter = MipFilter.Linear;
    int wrap = WrapMode.Repeat;
    int op = CompareOp.Gt;
};
