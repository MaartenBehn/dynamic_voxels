use envparse::parse_env;

pub const VOXELS_PER_METER: usize = parse_env!("VOXELS_PER_METER" as usize);
pub const METERS_PER_SHADER_UNIT: usize = parse_env!("METERS_PER_SHADER_UNIT" as usize);
pub const VOXELS_PER_SHADER_UNIT: usize = VOXELS_PER_METER * METERS_PER_SHADER_UNIT;

pub const GI_ATLAS_SIZE: usize = parse_env!("GI_ATLAS_SIZE" as usize);
pub const PROBE_RADIANCE_RES: usize = parse_env!("PROBE_RADIANCE_RES" as usize);
pub const PROBE_DEPTH_RES: usize = parse_env!("PROBE_DEPTH_RES" as usize);

pub const PROBE_PADDING: usize = 1;

