use std::error::Error;

use super::generate_seeds;

pub fn gen_ethyne_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let ethyne_table_name = "ethyne_path.yaml";
    generate_seeds(export_loc_str, potential_loc_str, edft, ethyne_table_name)
}
