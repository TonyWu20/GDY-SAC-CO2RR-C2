use std::{
    error::Error,
    fs::{create_dir_all, write},
    path::Path,
};

use castep_model_generator_backend::{
    atom::Atom,
    external_info::{
        element_table::{Element, ElmTab},
        YamlTable,
    },
    lattice::LatticeTraits,
    parser::msi_parser::MsiModel,
    MsiExport,
};
use nalgebra::Matrix3;

use crate::gdy_lattice::GDYLattice;

pub fn load_base_model() -> Result<GDYLattice, std::io::Error> {
    let basic_model_file = std::fs::read_to_string("./resources/SAC_GDY_M.msi")?;
    let msi_model: MsiModel = MsiModel::try_from(basic_model_file.as_str()).unwrap();
    let lattice = GDYLattice::from(msi_model);
    Ok(lattice)
}

impl From<MsiModel> for GDYLattice {
    fn from(item: MsiModel) -> Self {
        let lattice_vectors_raw: [[f64; 3]; 3] = item.lattice_vectors().unwrap();
        let vectors = lattice_vectors_raw.to_vec();
        let vectors: Vec<Vec<f64>> = vectors.iter().map(|arr| arr.to_vec()).collect();
        let vectors: Vec<f64> = vectors.concat();
        let lattice_vectors_matrix = Matrix3::from_vec(vectors);
        let atoms = item.atoms().to_owned();
        GDYLattice::new(
            "SAC_GDY_M".to_string(),
            lattice_vectors_matrix,
            atoms,
            73,
            false,
            None,
        )
    }
}

pub fn edit_metal(basic_model: &GDYLattice, element: &Element) -> GDYLattice {
    let element_id: u32 = element.atomic_number as u32;
    let element_name = element.element.to_owned();
    let metal_id = basic_model.metal_site();
    let mut new_model = basic_model.clone();
    let metal_atom: &mut Atom = new_model
        .get_mut_atoms()
        .get_mut((metal_id - 1) as usize)
        .unwrap();
    metal_atom.set_element_id(element_id);
    metal_atom.set_element_name(&element_name);
    let new_name = format!("SAC_GDY_{element_name}");
    new_model.set_lattice_name(new_name);
    new_model
}

pub fn generate_all_metal_models() -> Result<(), Box<dyn Error>> {
    let base = load_base_model()?;
    let metal_table = ElmTab::load_table("./resources/metal_table.yaml")?;
    let metals: Option<&Vec<Element>> = metal_table.elements();
    let metal_iter = metals.unwrap().into_iter();
    let new_lattices: Vec<GDYLattice> = metal_iter
        .map(|elm| -> GDYLattice { edit_metal(&base, &elm) })
        .collect();
    new_lattices
        .iter()
        .try_for_each(|lat| -> Result<(), Box<dyn Error>> {
            let lat_name = &lat.lattice_name();
            let export_dir = format!("./gdy_sac_models/{lat_name}");
            create_dir_all(&export_dir)?;
            let export_path = format!("{export_dir}/{lat_name}.msi");
            let export_text = lat.output_in_msi();
            write(Path::new(&export_path), export_text)?;
            Ok(())
        })?;
    Ok(())
}
