#![allow(dead_code, non_snake_case)]

use core::panic;
use std::{fs, str::FromStr};

use castep_model_core::{LatticeModel, ModelInfo, MsiModel};
use castep_model_generator_backend::adsorbate::{AdsInfo, Adsorbate};

#[cfg(test)]
mod test;

/// Trait to act as a `Pathway`
pub trait Pathway: Send + Sync {}

#[derive(Debug, Clone)]
/// Generic struct for Adsorption models from different pathway and in different format.
/// The struct has a lifetime as long as the `&AdsInfo`. The `&AdsInfo` is borrowed
/// from the `AdsTab` which we will load at the beginning of the workflow.
pub struct AdsModel<'a, T: ModelInfo, const N: usize> {
    lattice_model: LatticeModel<T>,
    ads_info: &'a AdsInfo,
}

impl<'a, T: ModelInfo, const N: usize> AdsModel<'a, T, N> {
    pub fn new(lattice_model: LatticeModel<T>, ads_info: &'a AdsInfo) -> Self {
        Self {
            lattice_model,
            ads_info,
        }
    }

    pub fn lattice_model(&self) -> &LatticeModel<T> {
        &self.lattice_model
    }

    pub fn ads_info(&self) -> &AdsInfo {
        self.ads_info
    }
}

impl<'a, T, const N: usize> Adsorbate for AdsModel<'a, T, N> where T: ModelInfo {}

impl<'a, const N: usize> AdsModel<'a, MsiModel, N> {
    pub fn load_model(source_dir: &str, ads_info: &'a AdsInfo) -> Self {
        let ads_name = ads_info.name();
        let cwd = env!("CARGO_MANIFEST_DIR");
        let filepath = format!("{}/{}/{}.msi", cwd, source_dir, ads_name);
        let msi_content = fs::read_to_string(&filepath)
            .unwrap_or_else(|_| panic!("{} does not exist!", &filepath));
        let msi_model: LatticeModel<MsiModel> =
            LatticeModel::from_str(msi_content.as_str()).unwrap();
        Self::new(msi_model, ads_info)
    }
}
