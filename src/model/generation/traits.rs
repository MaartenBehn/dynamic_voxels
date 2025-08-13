use crate::volume::{remove_trait::VolumeRemove, VolumeQureyPosValid, VolumeQureyPosValid2D};
use std::fmt::Debug;

pub trait IT: Debug + Copy + Eq + Default {}
pub trait BU: Debug + Clone + Default {}

pub trait ModelGenerationTypes: Debug + Clone + Default {
    type Identifier: IT; 
    type UndoData: BU;
    type Volume: VolumeQureyPosValid + VolumeRemove + Debug + Clone;
    type Volume2D: VolumeQureyPosValid2D + VolumeRemove + Debug + Clone;
} 
