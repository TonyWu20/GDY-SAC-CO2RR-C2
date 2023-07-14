use std::error::Error;

use adsorption_pathways::EthynePathway;

use super::GenerateSeeds;

impl GenerateSeeds for EthynePathway {
    type ThePathwayId = Self;
}

pub fn gen_ethyne_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let ethyne_table_name = "ethyne_path.yaml";
    EthynePathway::generate_seeds(export_loc_str, potential_loc_str, edft, ethyne_table_name)
}
