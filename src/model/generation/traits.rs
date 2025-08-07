use crate::volume::{VolumeQureyPosValid, VolumeQureyPosValid2D};
use std::fmt::Debug;

pub trait IT: Debug + Copy + Eq + Default {}
pub trait BU: Debug + Clone + Default {}

pub trait ModelGenerationTypes: Debug + Clone + Default {
    type Identifier: IT; 
    type UndoData: BU;
    type Volume: VolumeQureyPosValid;
    type Volume2D: VolumeQureyPosValid2D;
} 
