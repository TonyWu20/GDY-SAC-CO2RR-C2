use std::{
    error::Error,
    fs::{create_dir_all, write},
    path::Path,
};

use castep_model_generator_backend::{
    atom::Atom, lattice::LatticeModel, model_type::msi::MsiModel,
};

use crate::gdy_lattice::GDYLattice;

use cpt::{data::ELEMENT_TABLE, element::Element};

pub fn load_base_model() -> Result<GDYLattice<MsiModel>, std::io::Error> {
    let cwd = env!("CARGO_MANIFEST_DIR");
    let basic_model_file = std::fs::read_to_string(&format!("{}/../resources/SAC_GDY_M.msi", cwd))
        .unwrap_or_else(|e| panic!("Error when loading basic model, {e}"));
    let lattice = LatticeModel::try_from(basic_model_file.as_str()).unwrap();
    let gdy_base_lattice = GDYLattice::new(lattice, "SAC_GDY_M".to_string(), 73);
    Ok(gdy_base_lattice)
}

pub fn edit_metal(basic_model: &GDYLattice<MsiModel>, element: &Element) -> GDYLattice<MsiModel> {
    let element_id: u32 = element.atomic_number() as u32;
    let element_symbol = element.symbol();
    let metal_id = basic_model.metal_site();
    let mut new_model = basic_model.clone();
    let metal_atom: &mut Atom<MsiModel> = new_model
        .lattice_mut()
        .atoms_mut()
        .get_mut((metal_id - 1) as usize)
        .unwrap();
    metal_atom.set_element_id(element_id);
    metal_atom.set_element_symbol(element_symbol.to_string());
    let new_name = format!("SAC_GDY_{element_symbol}");
    new_model.set_lattice_name(new_name);
    new_model
}

pub fn generate_all_metal_models() -> Result<(), Box<dyn Error>> {
    let base = load_base_model()?;
    let metals = &ELEMENT_TABLE[2..];
    let new_lattices: Vec<GDYLattice<MsiModel>> = metals
        .iter()
        .map(|elm: &Element| -> GDYLattice<MsiModel> { edit_metal(&base, elm) })
        .collect();
    new_lattices.iter().try_for_each(
        |lat: &GDYLattice<MsiModel>| -> Result<(), Box<dyn Error>> {
            let lat_name = &lat.lattice_name();
            let export_dir = format!("./gdy_sac_models/{lat_name}");
            create_dir_all(&export_dir)?;
            let export_path = format!("{export_dir}/{lat_name}.msi");
            let msi_lat: LatticeModel<MsiModel> = lat.lattice().to_owned();
            let export_text = msi_lat.msi_export();
            write(Path::new(&export_path), export_text)?;
            Ok(())
        },
    )?;
    Ok(())
}
