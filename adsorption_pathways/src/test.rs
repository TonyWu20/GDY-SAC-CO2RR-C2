use castep_model_core::MsiModel;
use castep_model_generator_backend::external_info::{adsorbate_table::AdsTab, YamlTable};

use crate::{
    ethane_pathway::{CH2Pathway, COPathway},
    water_pathway::Water,
    AdsModel,
};
#[test]
fn parse_adsorbate() {
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
    let water_ads_table = AdsTab::load_table("water.yaml").unwrap();
    water_ads_table
        .adsorbates()
        .unwrap()
        .iter()
        .for_each(|ads_info| {
            let ads: AdsModel<MsiModel, Water> = AdsModel::from(ads_info);
            println!("{:#?}", ads);
        })
}
