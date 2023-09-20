use std::error::Error;

use super::generate_seeds;

pub fn gen_ethane_pathway_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let ch2_table_name = "ethane_ch2.yaml";
    let co_table_name = "ethane_co_dimer.yaml";
    generate_seeds(export_loc_str, potential_loc_str, edft, ch2_table_name)?;
    generate_seeds(export_loc_str, potential_loc_str, edft, co_table_name)
}
