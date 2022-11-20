#![allow(non_snake_case)]

use std::error::Error;

use crate::tasks::gen_ethane_pathway_seeds;

// use basic_models::gdy_model_edit::generate_all_metal_models;

mod tasks;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let cwd = env!("CARGO_MANIFEST_DIR");
    println!("{}", cwd);
    // generate_all_metal_models()?;
    gen_ethane_pathway_seeds()
}
#[cfg(test)]
mod test {}
