use std::fs;

use castep_model_generator_backend::{
    adsorbate::AdsInfo, lattice::LatticeModel, model_type::msi::MsiModel, model_type::ModelInfo,
};
#[derive(Debug, Clone)]
pub struct CH2Adsorbate<T: ModelInfo> {
    lattice_model: LatticeModel<T>,
    ads_info: AdsInfo,
}

impl<T: ModelInfo> CH2Adsorbate<T> {
    pub fn new(lattice_model: LatticeModel<T>, ads_info: AdsInfo) -> Self {
        Self {
            lattice_model,
            ads_info,
        }
    }
}

impl From<&AdsInfo> for CH2Adsorbate<MsiModel> {
    fn from(item: &AdsInfo) -> Self {
        let ads_name = item.name().to_owned();
        let cwd = env!("CARGO_MANIFEST_DIR");
        let filepath = format!("{}/adsorbates/CH2_coupling/{ads_name}.msi", cwd);
        let msi_content = fs::read_to_string(filepath).unwrap();
        let msi_model = LatticeModel::try_from(msi_content.as_str()).unwrap();
        Self::new(msi_model, item.to_owned())
    }
}
