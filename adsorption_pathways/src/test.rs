use castep_model_core::MsiModel;
use castep_model_generator_backend::external_info::{adsorbate_table::AdsTab, YamlTable};

use crate::AdsModel;
#[test]
fn parse_adsorbate() {
    let cwd = env!("CARGO_MANIFEST_DIR");
    println!("{}", cwd);
    let ads_table = AdsTab::load_table("ethane_ch2.yaml").unwrap();
    ads_table.adsorbates().unwrap().iter().for_each(|ads_info| {
        match ads_info.coord_atom_ids().len() {
            1 => {
                let ads =
                    AdsModel::<MsiModel, 1>::load_model(ads_table.source_directory(), ads_info);
                println!("{:#?}", ads);
            }
            2 => {
                let ads =
                    AdsModel::<MsiModel, 2>::load_model(ads_table.source_directory(), ads_info);
                println!("{:#?}", ads);
            }
            _ => {}
        }
    });
    let co_ads_table = AdsTab::load_table("ethane_co_dimer.yaml").unwrap();
    co_ads_table
        .adsorbates()
        .unwrap()
        .iter()
        .for_each(|ads_info| match ads_info.coord_atom_ids().len() {
            1 => {
                let ads =
                    AdsModel::<MsiModel, 1>::load_model(co_ads_table.source_directory(), ads_info);
                println!("{:#?}", ads);
            }
            2 => {
                let ads =
                    AdsModel::<MsiModel, 2>::load_model(co_ads_table.source_directory(), ads_info);
                println!("{:#?}", ads);
            }
            _ => {}
        });
    let water_ads_table = AdsTab::load_table("water.yaml").unwrap();
    water_ads_table
        .adsorbates()
        .unwrap()
        .iter()
        .for_each(|ads_info| match ads_info.coord_atom_ids().len() {
            1 => {
                let ads = AdsModel::<MsiModel, 1>::load_model(
                    water_ads_table.source_directory(),
                    ads_info,
                );
                println!("{:#?}", ads);
            }
            2 => {
                let ads = AdsModel::<MsiModel, 2>::load_model(
                    water_ads_table.source_directory(),
                    ads_info,
                );
                println!("{:#?}", ads);
            }
            _ => {}
        });
    let ethyne_tab = AdsTab::load_table("ethyne_path.yaml").unwrap();
    ethyne_tab
        .adsorbates()
        .unwrap()
        .iter()
        .for_each(|ads_info| match ads_info.coord_atom_ids().len() {
            1 => {
                let ads =
                    AdsModel::<MsiModel, 1>::load_model(ethyne_tab.source_directory(), ads_info);
                println!("{:#?}", ads);
            }
            2 => {
                let ads =
                    AdsModel::<MsiModel, 2>::load_model(ethyne_tab.source_directory(), ads_info);
                println!("{:#?}", ads);
            }
            _ => {}
        })
}
