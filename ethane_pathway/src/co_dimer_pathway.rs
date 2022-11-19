use std::fs;

use castep_model_generator_backend::{
    adsorbate::AdsInfo, lattice::LatticeModel, model_type::msi::MsiModel,
};

use crate::{AdsModel, Pathway};

#[derive(Debug)]
pub struct COPathway;
impl Pathway for COPathway {}

impl<'a> From<&'a AdsInfo> for AdsModel<'a, MsiModel, COPathway> {
    fn from(item: &'a AdsInfo) -> Self {
        let ads_name = item.name();
        let cwd = env!("CARGO_MANIFEST_DIR");
        let filepath = format!("{}/adsorbates/CO_dimer/{ads_name}.msi", cwd);
        let msi_content = fs::read_to_string(filepath).unwrap();
        let msi_model = LatticeModel::try_from(msi_content.as_str()).unwrap();
        Self::new(msi_model, item)
    }
}
