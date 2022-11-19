use std::fs;

use castep_model_generator_backend::{
    adsorbate::{AdsInfo, Adsorbate},
    lattice::LatticeModel,
    model_type::msi::MsiModel,
    model_type::ModelInfo,
};
#[derive(Debug, Clone)]
pub struct COAdsorbate<T: ModelInfo> {
    lattice_model: LatticeModel<T>,
    ads_info: AdsInfo,
}

/// Label it with the `Adsorbate` trait
impl<T: ModelInfo> Adsorbate for COAdsorbate<T> {}

impl<T: ModelInfo> COAdsorbate<T> {
    pub fn new(lattice_model: LatticeModel<T>, ads_info: AdsInfo) -> Self {
        Self {
            lattice_model,
            ads_info,
        }
    }
}
impl From<&AdsInfo> for COAdsorbate<MsiModel> {
    fn from(item: &AdsInfo) -> Self {
        let ads_name = item.name().to_owned();
        let cwd = env!("CARGO_MANIFEST_DIR");
        let filepath = format!("{}/adsorbates/CO_dimer/{ads_name}.msi", cwd);
        let msi_content = fs::read_to_string(filepath).unwrap();
        let msi_model = LatticeModel::try_from(msi_content.as_str())
            .unwrap_or_else(|e| panic!("{}: Error while parsing {}", e, ads_name));
        Self::new(msi_model, item.to_owned())
    }
}
