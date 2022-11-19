use std::{collections::HashMap, error::Error, path::Path};

use basic_models::{
    gdy_lattice::GDYLattice,
    gdy_model_edit::{edit_metal, load_base_model},
};
use castep_periodic_table::data::ELEMENT_TABLE;
use ethane_pathway::ch2_pathway::CH2Adsorbate;

const CWD: &str = env!("CARGO_MANIFEST_DIR");
