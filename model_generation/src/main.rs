#![allow(non_snake_case)]

use std::error::Error;

// use basic_models::gdy_model_edit::generate_all_metal_models;

use crate::tasks::add_ch2_pathway;

mod tasks;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let cwd = env!("CARGO_MANIFEST_DIR");
    println!("{}", cwd);
    // generate_all_metal_models()?;
    let ch2_table_path = format!("{cwd}/../ethane_pathway/ethane_pathway.yaml");
    add_ch2_pathway(&ch2_table_path)?;
    Ok(())
}
#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use basic_models::{gdy_lattice::GDYLattice, gdy_model_edit::edit_metal};
    use castep_model_generator_backend::{
        assemble::AddAdsorbate,
        external_info::{
            adsorbate_table::{AdsInfo, AdsTab},
            project::ProjectInfo,
            YamlTable,
        },
        lattice::LatticeTraits,
        param_writer::ParamWriter,
        parser::msi_parser::MsiModel,
    };
    use cpt::{data::ELEMENT_TABLE, element::LookupElement};
    use ethane_pathway::ch2_pathway::CH2Adsorbate;

    extern crate castep_periodic_table as cpt;

    #[test]
    fn parse_adsorbate() {
        use castep_model_generator_backend::external_info::{adsorbate_table::AdsTab, YamlTable};
        use ethane_pathway::ch2_pathway::CH2Adsorbate;

        let ads_table = AdsTab::load_table("../ethane_pathway/ethane_pathway.yaml")
            .unwrap_or_else(|e| panic!("Error while loading table, {:#?}", e));
        ads_table.adsorbates().unwrap().iter().for_each(|ads_info| {
            let ads = CH2Adsorbate::from(ads_info);
            println!("{:#?}", ads);
        });
    }
    #[test]
    fn add_ads_works() {
        let cwd = env!("CARGO_MANIFEST_DIR");
        let ads_tab = AdsTab::load_table(format!("{cwd}/../ethane_pathway/ethane_pathway.yaml"))
            .unwrap_or_else(|e| panic!("Error while loading ads_tab, {e}"));
        let first_ad: &AdsInfo = &ads_tab.adsorbates().unwrap()[0];
        let file = read_to_string(format!("{cwd}/../gdy_sac_models/SAC_GDY_Ag/SAC_GDY_Ag.msi"))
            .unwrap_or_else(|e| panic!("Error while loading msi, {e}"));
        let base_model = MsiModel::try_from(file.as_str()).unwrap();
        let ag_lat = GDYLattice::from(base_model);
        let mut ag_lat = edit_metal(&ag_lat, ELEMENT_TABLE.get_by_symbol("Ag").unwrap());
        let mut first_ad = CH2Adsorbate::from(first_ad);
        let project_tab = ProjectInfo::load_table(format!("{cwd}/../project.yaml")).unwrap();
        ag_lat
            .add_ads(
                &mut first_ad,
                &[41],
                1.4,
                true,
                &project_tab.hash_coord_site(),
            )
            .unwrap();
        ag_lat
            .write_seed_files(
                &format!("{cwd}/../gdy_sac_models"),
                &format!("{cwd}/../../C-GDY-SAC/Potentials"),
            )
            .unwrap();
    }
}
