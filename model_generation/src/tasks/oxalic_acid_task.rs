use std::error::Error;

use super::generate_seeds;

pub fn gen_oxalic_acid_seeds(
    export_loc_str: &str,
    potential_loc_str: &str,
    edft: bool,
) -> Result<(), Box<dyn Error>> {
    let oxalic_table_name = "oxalic_acid.yaml";
    generate_seeds(export_loc_str, potential_loc_str, edft, oxalic_table_name)
}
