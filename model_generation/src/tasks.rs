use std::{error::Error, io};

use basic_models::{
    gdy_lattice::{GDYLattice, GDY_COORD_SITES, GDY_DOUBLE_CASES, GDY_SINGLE_CASES},
    gdy_model_edit::generate_all_metal_models,
};
use castep_model_generator_backend::{
    adsorbate::AdsInfo,
    assemble::{AdsParamsBuilder, AdsorptionBuilder},
    builder_typestate::{No, Yes},
    external_info::{adsorbate_table::AdsTab, YamlTable},
    model_type::{cell::CellModel, msi::MsiModel, ModelInfo},
    param_writer::{
        castep_param::{BandStructureParam, GeomOptParam},
        seed_writer::SeedWriter,
    },
};
use rayon::prelude::*;

use ethane_pathway::{AdsModel, CH2Pathway, COPathway, Pathway};

const CWD: &str = env!("CARGO_MANIFEST_DIR");

fn build_ads_param<T>(
    gdy_latttice: &GDYLattice<T>,
    info: &AdsInfo,
    target_site_ids: &[u32],
) -> AdsParamsBuilder<Yes, Yes, Yes>
where
    T: ModelInfo,
{
    let plane_angle = if info.vertical() {
        90.0
    } else if info.name() == "OCC" {
        45.0
    } else {
        0.0
    };
    let coord_nums = info.coord_atom_ids().len();
    let ads_direction = if coord_nums == 1 {
        gdy_latttice.lattice().get_vector_ab(41, 42).unwrap()
    } else {
        gdy_latttice
            .lattice()
            .get_vector_ab(target_site_ids[0], target_site_ids[1])
            .unwrap()
    };
    AdsParamsBuilder::<No, No, No>::new()
        .with_plane_angle(plane_angle)
        .with_bond_length(1.4)
        .with_ads_direction(&ads_direction)
        .finish()
}

fn adsorption_naming<P: Pathway>(ads: &AdsModel<MsiModel, P>, target_site_ids: &[u32]) -> String {
    let ads_name = ads.ads_info().name();
    let site_names: Vec<String> = target_site_ids
        .iter()
        .map(|&site_id| {
            GDY_COORD_SITES
                .iter()
                .find(|cd_site| cd_site.site_id() == site_id)
                .unwrap()
                .site_name()
                .to_string()
        })
        .collect();
    let site_name_suffix = site_names.join("_");
    format!("_{ads_name}_{site_name_suffix}")
}

fn create_adsorption_at_site<P: Pathway>(
    gdy_lat: &GDYLattice<MsiModel>,
    ads: &AdsModel<MsiModel, P>,
    target_site_ids: &[u32],
) -> GDYLattice<CellModel> {
    let builder = AdsorptionBuilder::new(gdy_lat.lattice().to_owned());
    // Clone the `LatticeModel<MsiModel>` in the adsorbate.
    let ads_lattice = ads.lattice_model().to_owned();
    let ads_param = build_ads_param(&gdy_lat, ads.ads_info(), target_site_ids);
    let ads_info = ads.ads_info();
    // Builder actions
    let built_lattice = builder
        .add_adsorbate(ads_lattice)
        .with_location_at_sites(target_site_ids)
        .with_ads_params(ads_param)
        .align_ads(ads_info.stem_atom_ids())
        .init_ads_plane_direction(ads_info.plane_atom_ids())
        .place_adsorbate(ads_info.stem_atom_ids(), ads_info.coord_atom_ids(), 1.4)
        .build_adsorbed_lattice();
    let mut lat_name: String = gdy_lat.lattice_name().to_string();
    lat_name.push_str(&adsorption_naming(ads, target_site_ids));
    GDYLattice::new(built_lattice.into(), lat_name)
}

fn generate_seed_file(
    gdy_cell: &GDYLattice<CellModel>,
    export_loc_str: &str,
    potential_loc_str: &str,
) -> Result<(), io::Error> {
    let geom_seed_writer: SeedWriter<GeomOptParam> = SeedWriter::build(gdy_cell.lattice())
        .with_seed_name(gdy_cell.lattice_name())
        .with_export_loc(export_loc_str)
        .with_potential_loc(potential_loc_str)
        .build();
    geom_seed_writer.write_seed_files()?;
    // #[cfg(not(debug_assertions))]
    // {
    //     geom_seed_writer.copy_potentials()?;
    // }

    let bs_writer: SeedWriter<BandStructureParam> = geom_seed_writer.into();
    bs_writer.write_seed_files()?;
    Ok(())
}

fn iter_over_sites<P: Pathway>(
    gdy_lat: &GDYLattice<MsiModel>,
    adsorbate: &AdsModel<MsiModel, P>,
) -> Vec<GDYLattice<CellModel>> {
    let coord_atom_nums = adsorbate.ads_info().coord_atom_ids().len();
    let adsorbed_lats: Vec<GDYLattice<CellModel>> = if coord_atom_nums == 1 {
        GDY_SINGLE_CASES
            .iter()
            .map(|site| create_adsorption_at_site(gdy_lat, adsorbate, &[*site]))
            .collect()
    } else {
        if adsorbate.ads_info().symmetric() {
            GDY_DOUBLE_CASES
                .iter()
                .map(|sites| create_adsorption_at_site(gdy_lat, adsorbate, sites))
                .collect()
        } else {
            let mut asym: Vec<GDYLattice<CellModel>> = GDY_DOUBLE_CASES
                .iter()
                .map(|sites| create_adsorption_at_site(gdy_lat, adsorbate, sites))
                .collect();
            let asym_reverse: Vec<GDYLattice<CellModel>> = GDY_DOUBLE_CASES
                .iter()
                .map(|[site_1, site_2]| {
                    create_adsorption_at_site(gdy_lat, adsorbate, &[*site_2, *site_1])
                })
                .collect();
            asym.extend(asym_reverse);
            asym
        }
    };
    adsorbed_lats
}

fn iter_all_ads<'a, P>(
    gdy_lat: &'a GDYLattice<MsiModel>,
    ads_tab: &'a AdsTab,
    export_loc_str: &'a str,
    potential_loc_str: &'a str,
) where
    P: Pathway + 'static,
{
    ads_tab
        .adsorbates()
        .unwrap()
        .iter()
        .map(|ads_info| AdsModel::<MsiModel, P>::from(ads_info))
        .for_each(|ads| {
            iter_over_sites(gdy_lat, &ads).iter().for_each(|gdy_lat| {
                generate_seed_file(gdy_lat, export_loc_str, potential_loc_str).unwrap()
            })
        })
}

pub fn gen_ethane_pathway_seeds() -> Result<(), Box<dyn Error>> {
    let cwd = env!("CARGO_MANIFEST_DIR");
    let ch2_table_path = format!("{cwd}/../ethane_pathway/ethane_ch2.yaml");
    let co_table_path = format!("{cwd}/../ethane_pathway/ethane_co_dimer.yaml");
    let ch2_table = AdsTab::load_table(&ch2_table_path)?;
    let co_table = AdsTab::load_table(&co_table_path)?;
    let export_loc_str = "ethane_pathway_models";
    let potential_loc_str = format!("{cwd}/../../C-GDY-SAC/Potentials");
    generate_all_metal_models()
        .unwrap()
        .par_iter()
        .for_each(|gdy_lat| {
            iter_all_ads::<CH2Pathway>(
                &gdy_lat,
                &ch2_table,
                &format!("{}/{}", export_loc_str, "CH2_coupling"),
                &potential_loc_str,
            );
            iter_all_ads::<COPathway>(
                &gdy_lat,
                &co_table,
                &format!("{}/{}", export_loc_str, "CO_dimer"),
                &potential_loc_str,
            );
        });
    Ok(())
}
