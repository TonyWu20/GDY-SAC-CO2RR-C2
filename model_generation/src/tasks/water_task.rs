use std::error::Error;

use adsorption_pathways::Water;

use super::GenerateSeeds;

impl GenerateSeeds for Water {
    type ThePathwayId = Self;
}

pub fn gen_water_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let water_table_name = "water.yaml";
    Water::generate_seeds(export_loc_str, potential_loc_str, edft, water_table_name)
}
