use std::{
    error::Error,
    fs::{self, create_dir_all, read_to_string, rename},
    io,
};

use basic_models::{
    gdy_lattice::{GDYLattice, GDY_COORD_SITES, GDY_DOUBLE_CASES, GDY_SINGLE_CASES},
    gdy_model_edit::generate_all_metal_models,
};
use castep_model_core::{
    builder_typestate::No,
    param_writer::{
        castep_param::{BandStructureParam, GeomOptParam},
        ms_aux_files::to_xsd_scripts,
        seed_writer::SeedWriter,
    },
    CellModel, LatticeModel, ModelInfo, MsiModel,
};
use castep_model_generator_backend::{
    adsorbate::AdsInfo,
    assemble::{AdsParams, AdsParamsBuilder, AdsorptionBuilder},
    external_info::{adsorbate_table::AdsTab, YamlTable},
};
use castep_periodic_table::data::ELEMENT_TABLE;
use glob::glob;
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;

use ethane_pathway::{AdsModel, CH2Pathway, COPathway, Pathway};

const CWD: &str = env!("CARGO_MANIFEST_DIR");

fn build_ads_param<'a, T>(
    gdy_latttice: &GDYLattice<T>,
    info: &'a AdsInfo,
    target_site_ids: &[u32],
) -> AdsParams<'a>
where
    T: ModelInfo,
{
    let plane_angle = if let Some(angle) = info.plane_angle() {
        angle
    } else {
        0.0
    };
    let coord_angle = if let Some(angle) = info.stem_angle_at_coord() {
        angle
    } else {
        0.0
    };
    let plane_atom_ids = if let Some(ids) = info.plane_atom_ids() {
        ids.as_slice()
    } else {
        &[1]
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
    AdsParamsBuilder::<No, No, No, No>::new()
        .with_ads_direction(&ads_direction)
        .with_plane_angle(plane_angle)
        .with_stem_coord_angle(coord_angle)
        .with_bond_length(1.4)
        .with_stem_atom_ids(info.stem_atom_ids())
        .with_coord_atom_ids(info.coord_atom_ids())
        .with_plane_atom_ids(plane_atom_ids)
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
    let ads_param = build_ads_param(gdy_lat, ads.ads_info(), target_site_ids);
    // Builder actions
    let built_lattice = builder
        .add_adsorbate(ads_lattice)
        .with_location_at_sites(target_site_ids)
        .with_ads_params(ads_param)
        .init_ads(ads.ads_info().upper_atom_id())
        .place_adsorbate()
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
    copy_smcastep_extension(&geom_seed_writer)?;
    let bs_writer: SeedWriter<BandStructureParam> = geom_seed_writer.into();
    bs_writer.write_seed_files()?;
    Ok(())
}
fn copy_smcastep_extension(writer: &SeedWriter<GeomOptParam>) -> Result<(), io::Error> {
    let dest_dir = writer.create_export_dir()?;
    let with_seed_name = format!("SMCastep_Extension_{}.xms", writer.seed_name());
    let dest_path = dest_dir.join(&with_seed_name);
    if !dest_path.exists() {
        fs::copy(
            &format!("{}/../resources/SMCastep_Extension.xms", CWD),
            dest_path,
        )?;
    }
    Ok(())
}

fn iter_over_sites<P: Pathway>(
    gdy_lat: &GDYLattice<MsiModel>,
    adsorbate: &AdsModel<MsiModel, P>,
) -> Vec<GDYLattice<CellModel>> {
    let coord_atom_nums = adsorbate.ads_info().coord_atom_ids().len();
    let adsorbed_lats: Vec<GDYLattice<CellModel>> = if coord_atom_nums == 1 {
        GDY_SINGLE_CASES
            .par_iter()
            .map(|site| create_adsorption_at_site(gdy_lat, adsorbate, &[*site]))
            .collect()
    } else if adsorbate.ads_info().symmetric() {
        GDY_DOUBLE_CASES
            .par_iter()
            .map(|sites| create_adsorption_at_site(gdy_lat, adsorbate, sites))
            .collect()
    } else {
        let mut asym: Vec<GDYLattice<CellModel>> = GDY_DOUBLE_CASES
            .par_iter()
            .map(|sites| create_adsorption_at_site(gdy_lat, adsorbate, sites))
            .collect();
        let asym_reverse: Vec<GDYLattice<CellModel>> = GDY_DOUBLE_CASES
            .par_iter()
            .map(|[site_1, site_2]| {
                create_adsorption_at_site(gdy_lat, adsorbate, &[*site_2, *site_1])
            })
            .collect();
        asym.extend(asym_reverse);
        asym
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
        .par_iter()
        .map(AdsModel::<MsiModel, P>::from)
        .for_each(|ads| {
            iter_over_sites(gdy_lat, &ads).iter().for_each(|gdy_lat| {
                generate_seed_file(gdy_lat, export_loc_str, potential_loc_str).unwrap()
            })
        })
}

pub fn gen_ethane_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
) -> Result<(), Box<dyn Error>> {
    let cwd = env!("CARGO_MANIFEST_DIR");
    let ch2_table_path = format!("{cwd}/../ethane_pathway/ethane_ch2.yaml");
    let co_table_path = format!("{cwd}/../ethane_pathway/ethane_co_dimer.yaml");
    let ch2_table = AdsTab::load_table(&ch2_table_path)?;
    let co_table = AdsTab::load_table(&co_table_path)?;
    generate_all_metal_models()
        .unwrap()
        .par_iter()
        .progress()
        .for_each(|gdy_lat| {
            iter_all_ads::<CH2Pathway>(
                gdy_lat,
                &ch2_table,
                &format!("{}/{}", export_loc_str, "CH2_coupling"),
                &potential_loc_str,
            );
            iter_all_ads::<COPathway>(
                gdy_lat,
                &co_table,
                &format!("{}/{}", export_loc_str, "CO_dimer"),
                &potential_loc_str,
            );
        });
    to_xsd_scripts("ethane_pathway_models")?;
    Ok(())
}

pub fn post_copy_potentials(
    target_directory: &str,
    potential_loc_str: &str,
) -> Result<(), io::Error> {
    let msi_pattern = format!("{target_directory}/**/*.msi");
    let num_seeds = glob(&msi_pattern).unwrap().count();
    let bar = ProgressBar::new(num_seeds as u64);
    glob(&msi_pattern)
        .unwrap()
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| -> Result<(), io::Error> {
            let content = read_to_string(entry.as_ref().unwrap()).unwrap();
            let lat: LatticeModel<MsiModel> = LatticeModel::try_from(content.as_str()).unwrap();
            let cell: LatticeModel<CellModel> = lat.into();
            let filepath = entry.as_ref().unwrap().clone();
            let dir_path = filepath
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .clone();
            let cell_name = filepath.file_stem().unwrap().to_str().unwrap().to_owned();
            let writer: SeedWriter<GeomOptParam> = SeedWriter::build(&cell)
                .with_seed_name(&cell_name)
                .with_export_loc(dir_path)
                .with_potential_loc(potential_loc_str)
                .build();
            writer.copy_potentials()?;
            bar.inc(1);
            Ok(())
        })?;
    bar.finish();
    Ok(())
}

pub fn reorganize_folders(target_directory: &str) -> Result<(), io::Error> {
    let metal_elements = &ELEMENT_TABLE[3..];
    let num_seeds = glob(&format!("{}/**/SAC_GDY*opt", target_directory))
        .unwrap()
        .count();
    let bar = ProgressBar::new(num_seeds as u64);
    metal_elements.iter().try_for_each(|elm| {
        let metal_dir = format!("{}/{}", target_directory, elm.symbol());
        let metal_seeds_pattern = format!("{}/**/SAC_GDY_{}*_opt", target_directory, elm.symbol());
        create_dir_all(&metal_dir)?;
        glob(&metal_seeds_pattern)
            .unwrap()
            .into_iter()
            .par_bridge()
            .try_for_each(|entry| -> Result<(), io::Error> {
                let dir_path = entry.unwrap();
                let new_path = format!(
                    "{}/{}",
                    metal_dir,
                    dir_path
                        .components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_str()
                        .unwrap()
                );
                rename(dir_path, new_path)?;
                bar.inc(1);
                Ok(())
            })
    })?;
    Ok(bar.finish())
}
