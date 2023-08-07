use std::{
    any::TypeId,
    error::Error,
    fs::{self, create_dir_all, read_to_string, rename, write},
    io,
    process::Command,
    str::FromStr,
};

use basic_models::{
    gdy_lattice::{GDYLattice, GDY_COORD_SITES, GDY_DOUBLE_CASES, GDY_SINGLE_CASES},
    gdy_model_edit::generate_all_metal_models,
};
use castep_model_core::{
    atom::visitor::VisitCollection,
    builder_typestate::No,
    builder_typestate::Yes,
    param_writer::{
        castep_param::{BandStructureParam, GeomOptParam},
        ms_aux_files::to_xsd_scripts,
        seed_writer::{SeedWriter, SeedWriterBuilder},
    },
    CellModel, LatticeModel, ModelInfo, MsiModel,
};
use castep_model_generator_backend::{
    adsorbate::AdsInfo,
    assemble::{AdsParams, AdsParamsBuilder, AdsorptionBuilder},
    external_info::{adsorbate_table::AdsTab, YamlTable},
};
use castep_periodic_table::{data::ELEMENT_TABLE, element::Element};
use clap::ValueEnum;
use glob::glob;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;

use adsorption_pathways::{AdsModel, Pathway, PathwayId};

const CWD: &str = env!("CARGO_MANIFEST_DIR");

mod ethane_task;
mod ethyne_task;
mod ketene_task;
mod water_task;

pub use ethane_task::gen_ethane_pathway_seeds;
pub use ethyne_task::gen_ethyne_pathway_seeds;
pub use ketene_task::gen_ketene_pathway_seeds;
pub use water_task::gen_water_pathway_seeds;

/// Build the `AdsParams`
/// # Arguments:
/// - `gdy_lattice`: `&GDYLattice<T>` - reference to the host lattice,
/// supply the adsorbate direction vector.
/// - `ads_info`: `&'a AdsInfo` - reference to the adsorbate info object.
/// The lifetime of the returned `AdsParams` is bound to the passed `&AdsInfo`
/// - `target_site_ids`: `&[u32]` - slice, contains one or more atom ids. Used
/// to compute the adsorbate direction vector.
/// # To-do:
/// We need to handle cases where the `AdsInfo` attributes are none but the `AdsParam`
/// requires as the mandatory fields. This should be fixed in the crate
/// `castep-model-generator-backend`
fn build_ads_param<'a, T>(
    gdy_latttice: &GDYLattice<T>,
    ads_info: &'a AdsInfo,
    target_site_ids: &[u32],
) -> AdsParams<'a>
where
    T: ModelInfo,
{
    let coord_nums = ads_info.coord_atom_ids().len();
    let ads_direction = if ads_info.atom_nums() == 1 {
        None
    } else if ["CH", "CH2"].contains(&ads_info.name()) {
        Some(gdy_latttice.lattice().get_vector_ab(41, 73).unwrap())
    } else if coord_nums == 1 {
        Some(gdy_latttice.lattice().get_vector_ab(41, 42).unwrap())
    } else {
        Some(
            gdy_latttice
                .lattice()
                .get_vector_ab(target_site_ids[0], target_site_ids[1])
                .unwrap(),
        )
    };
    let bond_length = if let Some(value) = ads_info.bond_length() {
        value
    } else {
        1.4
    };
    AdsParamsBuilder::<No, No, No, No>::new()
        .with_ads_direction(ads_direction)
        .with_plane_angle(ads_info.plane_angle())
        .with_stem_coord_angle(ads_info.stem_angle_at_coord())
        .with_bond_length(bond_length)
        .with_stem_atom_ids(ads_info.stem_atom_ids())
        .with_coord_atom_ids(ads_info.coord_atom_ids())
        .with_plane_atom_ids(ads_info.plane_atom_ids())
        .finish()
}

/// Format the adsorption model name
/// It is `{lattice_name}_{ads_name}_{site_information}`
fn adsorption_naming<P: Pathway, const N: usize>(
    ads: &AdsModel<MsiModel, P, N>,
    target_site_ids: &[u32],
) -> String {
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

/// Iteration level: single lattice, single adsorbate, single coord case
fn create_adsorption_at_site<P: Pathway, const N: usize>(
    gdy_lat: &GDYLattice<MsiModel>,
    ads: &AdsModel<MsiModel, P, N>,
    coord_site_case: &[u32],
) -> GDYLattice<CellModel> {
    let builder = AdsorptionBuilder::new(gdy_lat.lattice().to_owned());
    // Clone the `LatticeModel<MsiModel>` in the adsorbate.
    let ads_lattice = ads.lattice_model().to_owned();
    let ads_param = build_ads_param(gdy_lat, ads.ads_info(), coord_site_case);
    // Builder actions
    let built_lattice = builder
        .add_adsorbate(ads_lattice)
        .with_location_at_sites(coord_site_case)
        .with_ads_params(ads_param)
        .init_ads(ads.ads_info().upper_atom_id())
        .place_adsorbate()
        .build_adsorbed_lattice();
    // Format the as-adsorbed model name
    let mut lat_name: String = gdy_lat.lattice_name().to_string();
    lat_name.push_str(&adsorption_naming(ads, coord_site_case));
    GDYLattice::new(built_lattice.into(), lat_name)
}

/// Iteration level: single lattice, single adsorbate, all coord cases on the lattice
/// ---iter_over_sites
/// ---|
/// --------->create_adsorption_at_site
trait IterOverSites<P: Pathway, const N: usize> {
    fn iter_over_sites(
        gdy_lat: &GDYLattice<MsiModel>,
        adsorbate: &AdsModel<MsiModel, P, N>,
    ) -> Vec<GDYLattice<CellModel>>;
}

/// Trait to implement placing two singly coord adsorbates in
/// the neighboring positions.
/// # Trait bound:
/// Super trait: `IterOverSites<P, 1>`
trait DoubleSingleCoord<P: Pathway>: IterOverSites<P, 1> {
    fn iter_neighbouring_sites(
        gdy_lat: &GDYLattice<MsiModel>,
        adsorbate: &AdsModel<MsiModel, P, 1>,
    ) -> Vec<GDYLattice<CellModel>>;
}

/// Unit struct to implement trait `IterOverSites` to mimick overloading
struct SitesIterator<const COORD_NUMS: usize>;
impl<P: Pathway> IterOverSites<P, 1> for SitesIterator<1> {
    fn iter_over_sites(
        gdy_lat: &GDYLattice<MsiModel>,
        adsorbate: &AdsModel<MsiModel, P, 1>,
    ) -> Vec<GDYLattice<CellModel>> {
        GDY_SINGLE_CASES
            .par_iter()
            .map(|site| create_adsorption_at_site(gdy_lat, adsorbate, &[*site]))
            .collect()
    }
}

impl<P: Pathway> DoubleSingleCoord<P> for SitesIterator<1> {
    fn iter_neighbouring_sites(
        gdy_lat: &GDYLattice<MsiModel>,
        adsorbate: &AdsModel<MsiModel, P, 1>,
    ) -> Vec<GDYLattice<CellModel>> {
        GDY_DOUBLE_CASES
            .par_iter()
            .map(|[site_1, site_2]| {
                let added_cell: GDYLattice<CellModel> =
                    create_adsorption_at_site(gdy_lat, adsorbate, &[*site_1]);
                let added_msi: GDYLattice<MsiModel> = added_cell.into();
                create_adsorption_at_site(&added_msi, adsorbate, &[*site_2])
            })
            .collect()
    }
}

impl<P: Pathway> IterOverSites<P, 2> for SitesIterator<2> {
    fn iter_over_sites(
        gdy_lat: &GDYLattice<MsiModel>,
        adsorbate: &AdsModel<MsiModel, P, 2>,
    ) -> Vec<GDYLattice<CellModel>> {
        let adsorbed_lats: Vec<GDYLattice<CellModel>> = if adsorbate.ads_info().symmetric() {
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
}

/// Final output
fn generate_seed_file(
    gdy_cell: &GDYLattice<CellModel>,
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), io::Error> {
    let geom_seed_writer: SeedWriterBuilder<GeomOptParam, Yes> =
        SeedWriter::build(gdy_cell.lattice())
            .with_seed_name(gdy_cell.lattice_name())
            .with_export_loc(export_loc_str)
            .with_potential_loc(potential_loc_str);
    let geom_seed_writer = if edft {
        geom_seed_writer.build_edft()
    } else {
        geom_seed_writer.build()
    };
    geom_seed_writer.write_seed_files()?;
    copy_smcastep_extension(&geom_seed_writer)?;
    let bs_writer: SeedWriter<BandStructureParam> = geom_seed_writer.into();
    bs_writer.write_seed_files()?;
    Ok(())
}

/// Copy the extension and rename to the model name.
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

pub trait GenerateSeeds {
    type ThePathwayId: 'static + Pathway;
    fn generate_seeds<'a>(
        export_loc_str: &'a str,
        potential_loc_str: &'a str,
        edft: bool,
        table_name: &'a str,
    ) -> Result<(), Box<dyn Error>> {
        let cwd = env!("CARGO_MANIFEST_DIR");
        let table_full_path = format!("{cwd}/../adsorption_pathways/{}", table_name);
        let table = AdsTab::load_table(table_full_path)?;
        let pathway_id = PathwayId::new(TypeId::of::<Self::ThePathwayId>());
        generate_all_metal_models()
            .unwrap()
            .par_iter()
            .progress()
            .for_each(|gdy_lat| {
                let metal_atomic_number = gdy_lat
                    .lattice()
                    .view_atom_by_id(gdy_lat.metal_site())
                    .unwrap()
                    .atomic_number()
                    .to_owned();
                let use_edft = if (57..=71).contains(&metal_atomic_number) {
                    edft
                } else {
                    false
                };
                iter_all_ads::<Self::ThePathwayId>(
                    gdy_lat,
                    &table,
                    &format!("{}/{}", export_loc_str, pathway_id.target_dir().unwrap()),
                    &potential_loc_str,
                    use_edft,
                );
            });
        let relative_dest = export_loc_str.split("/").last().unwrap();
        to_xsd_scripts(relative_dest)?;
        Ok(())
    }
}

fn iter_all_ads<'a, P>(
    gdy_lat: &'a GDYLattice<MsiModel>,
    ads_tab: &'a AdsTab,
    export_loc_str: &'a str,
    potential_loc_str: &'a str,
    edft: bool,
) where
    P: Pathway + 'static,
{
    ads_tab
        .adsorbates()
        .unwrap()
        .par_iter()
        .for_each(|ads_info| {
            let coord_num = ads_info.coord_atom_ids().len();
            match coord_num {
                1 => {
                    let ads = AdsModel::<MsiModel, P, 1>::from(ads_info);
                    SitesIterator::<1>::iter_over_sites(gdy_lat, &ads)
                        .par_iter()
                        .for_each(|gdy_lat| {
                            generate_seed_file(gdy_lat, export_loc_str, potential_loc_str, edft)
                                .unwrap()
                        });
                    if ["CH", "CH2"].contains(&ads.ads_info().name()) {
                        SitesIterator::<1>::iter_neighbouring_sites(gdy_lat, &ads)
                            .par_iter()
                            .for_each(|gdy_lat| {
                                generate_seed_file(gdy_lat, export_loc_str, potential_loc_str, edft)
                                    .unwrap()
                            })
                    }
                }
                2 => {
                    let ads = AdsModel::<MsiModel, P, 2>::from(ads_info);
                    SitesIterator::<2>::iter_over_sites(gdy_lat, &ads)
                        .par_iter()
                        .for_each(|gdy_lat| {
                            generate_seed_file(gdy_lat, export_loc_str, potential_loc_str, edft)
                                .unwrap()
                        })
                }
                _ => (),
            }
        });
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
            let lat: LatticeModel<MsiModel> = LatticeModel::from_str(content.as_str()).unwrap();
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
    let metal_elements = available_metals();
    let num_seeds = glob(&format!("{}/**/SAC_GDY*opt", target_directory))
        .unwrap()
        .count();
    let parent = glob(&format!("{}/**/SAC_GDY*opt", target_directory))
        .unwrap()
        .nth(0)
        .unwrap()
        .as_ref()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
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
    bar.finish();
    Command::new("rm")
        .args(["-r", &format!("{}", parent)])
        .output()
        .expect(&format!("Error when deleting the {} dir", parent));
    let relative_dest = target_directory.split("/").last().unwrap();
    to_xsd_scripts(relative_dest).unwrap();
    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ServerScriptType {
    PBS,
    LSF,
}
pub fn batch_submission_script(
    target_directory: &str,
    script_type: ServerScriptType,
) -> Result<(), io::Error> {
    let script_type = match script_type {
        ServerScriptType::LSF => "bsub MS70_YW_CASTEP.lsf",
        ServerScriptType::PBS => "qsub hpc.pbs.sh",
    };
    let script = format!(
        r#"#!/bin/sh
for i in `find . -maxdepth 1 -mindepth 1 -type d`; do
    cd $i
    {script_type} && cd ..
done"#
    );
    let metal_elements = available_metals();
    metal_elements.iter().try_for_each(|elm| {
        let metal_dir = format!("{}/{}", target_directory, elm.symbol());
        let script_path = format!("{metal_dir}/batch_submit.sh");
        write(script_path, &script)
    })
}

fn available_metals() -> Vec<Element> {
    let d3_metals = &ELEMENT_TABLE[20..30];
    let d4_metals = &ELEMENT_TABLE[38..48];
    let d5_metals = &ELEMENT_TABLE[56..80];
    vec![d3_metals, d4_metals, d5_metals].concat()
}
