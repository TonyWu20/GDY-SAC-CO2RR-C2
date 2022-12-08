#![allow(dead_code, non_snake_case)]

use std::{any::TypeId, fs, marker::PhantomData};

use castep_model_core::{LatticeModel, ModelInfo, MsiModel};
use castep_model_generator_backend::adsorbate::{AdsInfo, Adsorbate};
pub mod ethane_pathway;
pub mod ethyne_pathway;
pub mod water_pathway;

pub use ethane_pathway::{CH2Pathway, COPathway};
pub use ethyne_pathway::EthynePathway;
pub use water_pathway::Water;

#[cfg(test)]
mod test;

/// Trait to act as a `Pathway`
pub trait Pathway: Send + Sync {}

#[non_exhaustive]
#[derive(PartialEq, Eq)]
pub enum PathwayId {
    CH2,
    CO,
    Water,
    Ethyne,
    Undefined,
}

impl PathwayId {
    pub fn new(type_id: TypeId) -> Self {
        if type_id == TypeId::of::<CH2Pathway>() {
            Self::CH2
        } else if type_id == TypeId::of::<COPathway>() {
            Self::CO
        } else if type_id == TypeId::of::<Water>() {
            Self::Water
        } else if type_id == TypeId::of::<EthynePathway>() {
            Self::Ethyne
        } else {
            Self::Undefined
        }
    }
    fn source_directory(&self) -> Option<&str> {
        match *self {
            Self::CH2 => Some("CH2_coupling"),
            Self::CO => Some("CO_dimer"),
            Self::Water => Some("Water"),
            Self::Ethyne => Some("ethyne"),
            Self::Undefined => None,
        }
    }
    pub fn target_dir(&self) -> Option<&str> {
        match *self {
            Self::CH2 => Some("CH2_coupling"),
            Self::CO => Some("CO_dimer"),
            Self::Water => Some("Water"),
            Self::Ethyne => Some("ethyne"),
            Self::Undefined => None,
        }
    }
}

#[derive(Debug, Clone)]
/// Generic struct for Adsorption models from different pathway and in different format.
/// The struct has a lifetime as long as the `&AdsInfo`. The `&AdsInfo` is borrowed
/// from the `AdsTab` which we will load at the beginning of the workflow.
pub struct AdsModel<'a, T: ModelInfo, P: Pathway, const N: usize> {
    lattice_model: LatticeModel<T>,
    ads_info: &'a AdsInfo,
    pathway: PhantomData<P>,
}

impl<'a, T: ModelInfo, P: Pathway, const N: usize> AdsModel<'a, T, P, N> {
    pub fn new(lattice_model: LatticeModel<T>, ads_info: &'a AdsInfo) -> Self {
        Self {
            lattice_model,
            ads_info,
            pathway: PhantomData,
        }
    }

    pub fn lattice_model(&self) -> &LatticeModel<T> {
        &self.lattice_model
    }

    pub fn ads_info(&self) -> &AdsInfo {
        self.ads_info
    }
}

impl<'a, T, P, const N: usize> Adsorbate for AdsModel<'a, T, P, N>
where
    T: ModelInfo,
    P: Pathway,
{
}
impl<'a, P, const N: usize> From<&'a AdsInfo> for AdsModel<'a, MsiModel, P, N>
where
    P: Pathway + 'static,
{
    fn from(item: &'a AdsInfo) -> Self {
        let ads_name = item.name();
        let cwd = env!("CARGO_MANIFEST_DIR");
        let path_type_id = PathwayId::new(TypeId::of::<P>());
        let pathway_dir = path_type_id.source_directory().unwrap();
        let filepath = format!("{}/adsorbates/{}/{}.msi", cwd, pathway_dir, ads_name);
        let msi_content = fs::read_to_string(filepath).unwrap();
        let msi_model = LatticeModel::try_from(msi_content.as_str()).unwrap();
        Self::new(msi_model, item)
    }
}
