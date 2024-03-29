use std::{
    error::Error,
    fs::{create_dir_all, write},
    path::Path,
    str::FromStr,
};

use castep_model_core::{model_type::DefaultExport, LatticeModel, MsiModel};

use crate::gdy_lattice::GDYLattice;

use cpt::{data::ELEMENT_TABLE, element::Element};

pub fn load_base_model() -> Result<GDYLattice<MsiModel>, std::io::Error> {
    let cwd = env!("CARGO_MANIFEST_DIR");
    let basic_model_file = std::fs::read_to_string(format!("{}/../resources/SAC_GDY_M.msi", cwd))
        .unwrap_or_else(|e| panic!("Error when loading basic model, {e}"));
    let lattice = LatticeModel::from_str(basic_model_file.as_str()).unwrap();
    let gdy_base_lattice = GDYLattice::new(lattice, "SAC_GDY_M".to_string());
    Ok(gdy_base_lattice)
}

pub fn edit_metal(basic_model: &GDYLattice<MsiModel>, element: &Element) -> GDYLattice<MsiModel> {
    let atomic_number: u8 = element.atomic_number();
    let element_symbol = element.symbol();
    let metal_id = basic_model.metal_site();
    let mut new_model = basic_model.clone();
    new_model
        .lattice_mut()
        .atoms_mut()
        .update_symbol_at((metal_id - 1) as usize, element_symbol)
        .unwrap();
    new_model
        .lattice_mut()
        .atoms_mut()
        .update_elm_id_at((metal_id - 1) as usize, atomic_number)
        .unwrap();
    let new_name = format!("SAC_GDY_{element_symbol}");
    new_model.set_lattice_name(new_name);
    new_model
}

pub fn generate_all_metal_models() -> Result<Vec<GDYLattice<MsiModel>>, std::io::Error> {
    let base = load_base_model()?;
    let metals = available_metals();
    Ok(metals
        .iter()
        .map(|elm: &Element| -> GDYLattice<MsiModel> { edit_metal(&base, elm) })
        .collect::<Vec<GDYLattice<MsiModel>>>())
}

pub fn write_all_metal_models() -> Result<(), Box<dyn Error>> {
    let base = load_base_model()?;
    let metals = available_metals();
    metals
        .iter()
        .map(|elm: &Element| -> GDYLattice<MsiModel> { edit_metal(&base, elm) })
        .collect::<Vec<GDYLattice<MsiModel>>>()
        .iter()
        .try_for_each(|lat: &GDYLattice<MsiModel>| -> Result<(), Box<dyn Error>> {
            let lat_name = &lat.lattice_name();
            let export_dir = format!("./gdy_sac_models/{lat_name}");
            create_dir_all(&export_dir)?;
            let export_path = format!("{export_dir}/{lat_name}.msi");
            let msi_lat: LatticeModel<MsiModel> = lat.lattice().to_owned();
            let export_text = msi_lat.export();
            write(Path::new(&export_path), export_text)?;
            Ok(())
        })?;
    Ok(())
}

fn available_metals() -> Vec<Element> {
    let d3_metals = &ELEMENT_TABLE[20..30];
    let d4_metals = &ELEMENT_TABLE[38..48];
    let d5_metals = &ELEMENT_TABLE[56..80];
    vec![d3_metals, d4_metals, d5_metals].concat()
}

#[cfg(test)]
mod test {
    use super::available_metals;

    #[test]
    fn test_metal_ranges() {
        let metals = available_metals();
        metals.iter().for_each(|m| println!("{}", m.symbol()))
    }
}
