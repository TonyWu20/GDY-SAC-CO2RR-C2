use std::error::Error;

use super::generate_seeds;

pub fn gen_water_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let water_table_name = "water.yaml";
    generate_seeds(export_loc_str, potential_loc_str, edft, water_table_name)
}
