use std::error::Error;

use adsorption_pathways::ethane_pathway::{CH2Pathway, COPathway};

use super::GenerateSeeds;

impl GenerateSeeds for CH2Pathway {
    type ThePathwayId = CH2Pathway;
}

impl GenerateSeeds for COPathway {
    type ThePathwayId = COPathway;
}

pub fn gen_ethane_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let ch2_table_name = "ethane_ch2.yaml";
    let co_table_name = "ethane_co_dimer.yaml";
    CH2Pathway::generate_seeds(export_loc_str, potential_loc_str, edft, ch2_table_name)?;
    COPathway::generate_seeds(export_loc_str, potential_loc_str, edft, co_table_name)
}
