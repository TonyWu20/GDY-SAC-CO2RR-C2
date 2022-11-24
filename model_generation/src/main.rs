#![allow(non_snake_case)]

use std::error::Error;

use crate::tasks::{gen_ethane_pathway_seeds, post_copy_potentials};
use clap::{Parser, ValueEnum};

// use basic_models::gdy_model_edit::generate_all_metal_models;

mod tasks;

#[derive(Parser)]
#[command(author,version,about, long_about = None)]
struct Args {
    /// Target directory
    #[arg(short, long)]
    dir: Option<String>,
    #[arg(short, long)]
    potentials_loc: Option<String>,
    #[arg(short, long)]
    mode: Option<Mode>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Generate seed files without copying potentials
    Fast,
    /// Generate seed files and copy potentials
    Full,
    /// Copy potentials after seed files generation
    Post,
    /// Debug
    Debug,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    let cwd = env!("CARGO_MANIFEST_DIR");
    // generate_all_metal_models()?;
    let target_dir = cli.dir.as_ref();
    let target_dir_path = match target_dir {
        Some(dir) => format!("{}/../{}", cwd, dir),
        None => format!("{}/../ethane_pathway_models", cwd),
    };
    let pot_loc = cli.potentials_loc.as_ref();
    let potential_loc_path = match pot_loc {
        Some(dir) => format!("{}/../{}", cwd, dir),
        None => format!("{}/../Potentials", cwd),
    };
    let mode = cli.mode.as_ref();
    match mode {
        Some(m) => match m {
            Mode::Debug => {
                println!("{}", cwd);
                println!("{}", target_dir_path);
                println!("{}", potential_loc_path);
            }
            Mode::Fast => {
                gen_ethane_pathway_seeds()?;
            }
            _ => {
                gen_ethane_pathway_seeds()?;
                post_copy_potentials(&target_dir_path, &potential_loc_path)?;
            }
        },
        None => {
            gen_ethane_pathway_seeds()?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod test {}
