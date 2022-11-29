#![allow(dead_code, non_snake_case)]

use std::{any::TypeId, fs, marker::PhantomData};

use castep_model_core::{LatticeModel, ModelInfo, MsiModel};
use castep_model_generator_backend::adsorbate::{AdsInfo, Adsorbate};

pub trait Pathway: Send + Sync {}

#[derive(Debug)]
/// Marker struct to represent "CH2 coupling pathway"
pub struct CH2Pathway;
impl Pathway for CH2Pathway {}

#[derive(Debug)]
/// Marker struct to represent "CO dimerization pathway"
pub struct COPathway;
impl Pathway for COPathway {}

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

    pub fn lattice_model(&self) -> &LatticeModel<T> {
        &self.lattice_model
    }

    pub fn ads_info(&self) -> &AdsInfo {
        self.ads_info
    }
}

impl<'a, T, P> Adsorbate for AdsModel<'a, T, P>
where
    T: ModelInfo,
    P: Pathway,
{
}
impl<'a, P> From<&'a AdsInfo> for AdsModel<'a, MsiModel, P>
where
    P: Pathway + 'static,
{
    fn from(item: &'a AdsInfo) -> Self {
        let ads_name = item.name();
        let cwd = env!("CARGO_MANIFEST_DIR");
        let path_type_id = TypeId::of::<P>();
        let pathway_dir = if path_type_id == TypeId::of::<CH2Pathway>() {
            "CH2_coupling"
        } else if path_type_id == TypeId::of::<COPathway>() {
            "CO_dimer"
        } else {
            panic!("Wrong pathway type!")
        };
        let filepath = format!("{}/adsorbates/{}/{}.msi", cwd, pathway_dir, ads_name);
        let msi_content = fs::read_to_string(filepath).unwrap();
        let msi_model = LatticeModel::try_from(msi_content.as_str()).unwrap();
        Self::new(msi_model, item)
    }
}

#[cfg(test)]
#[test]
fn parse_adsorbate() {
    use castep_model_core::MsiModel;
    use castep_model_generator_backend::external_info::{adsorbate_table::AdsTab, YamlTable};

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
