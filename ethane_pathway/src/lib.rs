#![allow(dead_code, non_snake_case)]

pub mod ch2_pathway;
pub mod co_dimer_pathway;

#[cfg(test)]
#[test]
fn parse_adsorbate() {
    use castep_model_generator_backend::external_info::{adsorbate_table::AdsTab, YamlTable};

    use crate::{ch2_pathway::CH2Adsorbate, co_dimer_pathway::COAdsorbate};

    let cwd = env!("CARGO_MANIFEST_DIR");
    println!("{}", cwd);
    let ads_table = AdsTab::load_table("ethane_pathway.yaml").unwrap();
    ads_table.adsorbates().unwrap().iter().for_each(|ads_info| {
        let ads = CH2Adsorbate::from(ads_info);
        println!("{:#?}", ads);
    });
    let co_ads_table = AdsTab::load_table("ethane_co_dimer.yaml").unwrap();
    co_ads_table
        .adsorbates()
        .unwrap()
        .iter()
        .for_each(|ads_info| {
            let ads = COAdsorbate::from(ads_info);
            println!("{:#?}", ads);
        });
}
