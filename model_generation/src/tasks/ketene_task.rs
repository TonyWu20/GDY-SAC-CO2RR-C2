use std::error::Error;

use adsorption_pathways::ketene::KetenePathway;

use super::GenerateSeeds;

impl GenerateSeeds for KetenePathway {
    type ThePathwayId = Self;
}

pub fn gen_ketene_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let ketene_table_name = "ketene_others.yaml";
    KetenePathway::generate_seeds(export_loc_str, potential_loc_str, edft, ketene_table_name)
}
