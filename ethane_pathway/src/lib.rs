#![allow(dead_code, non_snake_case)]

use std::marker::PhantomData;

use castep_model_generator_backend::{
    adsorbate::{AdsInfo, Adsorbate},
    lattice::LatticeModel,
    model_type::ModelInfo,
};

pub mod ch2_pathway;
pub mod co_dimer_pathway;

pub trait Pathway {}

#[derive(Debug, Clone)]
/// Generic struct for Adsorption models from different pathway and in different format.
/// The struct has a lifetime as long as the `&AdsInfo`. The `&AdsInfo` is borrowed
/// from the `AdsTab` which we will load at the beginning of the workflow.
pub struct AdsModel<'a, T: ModelInfo, P: Pathway> {
    lattice_model: LatticeModel<T>,
    ads_info: &'a AdsInfo,
    pathway: PhantomData<P>,
}

impl<'a, T: ModelInfo, P: Pathway> AdsModel<'a, T, P> {
    pub fn new(lattice_model: LatticeModel<T>, ads_info: &'a AdsInfo) -> Self {
        Self {
            lattice_model,
            ads_info,
            pathway: PhantomData,
        }
    }
}

impl<'a, T, P> Adsorbate for AdsModel<'a, T, P>
where
    T: ModelInfo,
    P: Pathway,
{
}

#[cfg(test)]
#[test]
fn parse_adsorbate() {
    use castep_model_generator_backend::{
        external_info::{adsorbate_table::AdsTab, YamlTable},
        model_type::msi::MsiModel,
    };

    use crate::{ch2_pathway::CH2Pathway, co_dimer_pathway::COPathway};

    let cwd = env!("CARGO_MANIFEST_DIR");
    println!("{}", cwd);
    let ads_table = AdsTab::load_table("ethane_ch2.yaml").unwrap();
    ads_table.adsorbates().unwrap().iter().for_each(|ads_info| {
        let ads = AdsModel::<MsiModel, CH2Pathway>::from(ads_info);
        println!("{:#?}", ads);
    });
    let co_ads_table = AdsTab::load_table("ethane_co_dimer.yaml").unwrap();
    co_ads_table
        .adsorbates()
        .unwrap()
        .iter()
        .for_each(|ads_info| {
            let ads: AdsModel<MsiModel, COPathway> = AdsModel::from(ads_info);
            println!("{:#?}", ads);
        });
}
