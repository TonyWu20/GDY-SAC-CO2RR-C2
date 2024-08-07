#![allow(non_snake_case)]

use std::{error::Error, path::Path, process::Command};

use crate::tasks::{gen_ethane_pathway_seeds, post_copy_potentials};
use clap::{Parser, ValueEnum};
use tasks::{
    batch_submission_script, gen_ethyne_pathway_seeds, gen_ketene_pathway_seeds,
    gen_oxalic_acid_seeds, gen_water_pathway_seeds, reorganize_folders, ServerScriptType,
};

// use basic_models::gdy_model_edit::generate_all_metal_models;

mod tasks;

#[derive(Parser)]
#[command(author,version,about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pathway: Option<Pathway>,
    // Target directory
    #[arg(short, long)]
    dir: Option<String>,
    #[arg(long)]
    potentials_loc: Option<String>,
    #[arg(short, long)]
    mode: Option<Mode>,
    #[arg(long)]
    edft: Option<bool>,
    #[arg(long)]
    script: Option<ServerScriptType>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Pathway {
    Ethane,
    Ethyne,
    Water,
    Ketene,
    OxalicAcid,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Generate seed files without copying potentials
    Fast,
    /// Generate seed files and copy potentials
    Full,
    /// Copy potentials after seed files generation
    Post,
    /// Reorganize
    Reorg,
    /// Debug
    Debug,
    /// Clean the generated folder
    Clean,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    let cwd = env!("CARGO_MANIFEST_DIR");
    let pathway = if let Some(p) = cli.pathway {
        p
    } else {
        Pathway::Ethane
    };
    let pathway_string = match pathway {
        Pathway::Ethane => "ethane_pathway_models",
        Pathway::Ethyne => "ethyne_pathway_models",
        Pathway::Water => "water_pathway_models",
        Pathway::Ketene => "ketene_others_models",
        Pathway::OxalicAcid => "oxalic_acid_models",
    };
    // generate_all_metal_models()?;
    let target_dir = cli.dir.as_ref();
    let target_dir_path = match target_dir {
        Some(dir) => format!("{}/../{}", cwd, dir),
        None => format!("{}/../{}", cwd, pathway_string),
    };
    let pot_loc = cli.potentials_loc.as_ref();
    let potential_loc_path = match pot_loc {
        Some(dir) => format!("{}/../{}", cwd, dir),
        None => format!("{}/../Potentials", cwd),
    };
    let mode = cli.mode.as_ref();
    let edft = cli.edft.unwrap_or(true); // Default to use edft for rare earth
    let script_type = cli.script.unwrap_or(ServerScriptType::Pbs);
    match mode {
        Some(m) => match m {
            Mode::Debug => {
                println!("{}", cwd);
                println!("{:?}", pathway);
                println!("{}", target_dir_path);
                println!("{}", potential_loc_path);
            }
            Mode::Fast => {
                match pathway {
                    Pathway::Ethane => {
                        gen_ethane_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
                    }
                    Pathway::Ethyne => {
                        gen_ethyne_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
                    }
                    Pathway::Water => {
                        gen_water_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
                    }
                    Pathway::Ketene => {
                        gen_ketene_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
                    }
                    Pathway::OxalicAcid => {
                        gen_oxalic_acid_seeds(&target_dir_path, &potential_loc_path, edft)?
                    }
                };
                batch_submission_script(&target_dir_path, script_type, false)?
            }
            Mode::Reorg => {
                reorganize_folders(&target_dir_path)?;
                batch_submission_script(&target_dir_path, script_type, true)?
            }
            Mode::Post => {
                post_copy_potentials(&target_dir_path, &potential_loc_path)?;
            }
            Mode::Full => {
                gen_ethane_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?;
                gen_ethyne_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?;
                gen_water_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?;
                gen_ketene_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?;
                gen_oxalic_acid_seeds(&target_dir_path, &potential_loc_path, edft)?;
                post_copy_potentials(&target_dir_path, &potential_loc_path)?;
            }
            Mode::Clean => {
                if Path::new("ethane_pathway_models").exists() {
                    Command::new("rm")
                        .args(["-r", "ethane_pathway_models"])
                        .output()
                        .expect("Error while deleting 'ethane_pathway_models'");
                }
                if Path::new("ethyne_pathway_models").exists() {
                    Command::new("rm")
                        .args(["-r", "ethyne_pathway_models"])
                        .output()
                        .expect("Error while deleting 'ethyne_pathway_models'");
                }
                if Path::new("water_pathway_models").exists() {
                    Command::new("rm")
                        .args(["-r", "water_pathway_models"])
                        .output()
                        .expect("Error while deleting 'water_pathway_models'");
                }
                if Path::new("ketene_others_models").exists() {
                    Command::new("rm")
                        .args(["-r", "ketene_others_models"])
                        .output()
                        .expect("Error while deleting 'ketene_others_models'");
                }
                if Path::new("oxalic_acid").exists() {
                    Command::new("rm")
                        .args(["-r", "oxalic_acid"])
                        .output()
                        .expect("Error while deleting 'oxalic_acid'");
                }
            }
        },
        None => match pathway {
            Pathway::Ethane => {
                gen_ethane_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
            }
            Pathway::Ethyne => {
                gen_ethyne_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
            }
            Pathway::Water => gen_water_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?,
            Pathway::Ketene => {
                gen_ketene_pathway_seeds(&target_dir_path, &potential_loc_path, edft)?
            }
            Pathway::OxalicAcid => {
                gen_oxalic_acid_seeds(&target_dir_path, &potential_loc_path, edft)?
            }
        },
    }
    Ok(())
}
