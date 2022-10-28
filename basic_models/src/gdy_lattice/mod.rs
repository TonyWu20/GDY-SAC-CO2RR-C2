use castep_model_generator_backend::{atom::Atom, lattice::LatticeTraits, MsiExport};
use na::Matrix3;

extern crate nalgebra as na;
#[derive(Clone)]
pub struct GDYLattice {
    lattice_name: String,
    lattice_vectors: Matrix3<f64>,
    atoms: Vec<Atom>,
    metal_site: u32,
    sorted: bool,
    pathway: Option<String>,
}

impl GDYLattice {
    pub fn new(
        lattice_name: String,
        lattice_vectors: Matrix3<f64>,
        atoms: Vec<Atom>,
        metal_site: u32,
        sorted: bool,
        pathway: Option<String>,
    ) -> Self {
        Self {
            lattice_name,
            lattice_vectors,
            atoms,
            metal_site,
            sorted,
            pathway,
        }
    }

    pub fn set_lattice_name(&mut self, lattice_name: String) {
        self.lattice_name = lattice_name;
    }

    pub fn metal_site(&self) -> u32 {
        self.metal_site
    }

    pub fn lattice_name(&self) -> &str {
        self.lattice_name.as_ref()
    }
}

impl LatticeTraits for GDYLattice {
    fn get_lattice_vectors(&self) -> &Matrix3<f64> {
        &self.lattice_vectors
    }

    fn get_atoms(&self) -> &[Atom] {
        self.atoms.as_ref()
    }

    fn get_mut_atoms(&mut self) -> &mut Vec<Atom> {
        self.atoms.as_mut()
    }

    fn set_atoms_sorted(&mut self, is_sorted: bool) {
        self.sorted = is_sorted;
    }

    fn is_atoms_sorted(&self) -> bool {
        self.sorted
    }
}

impl MsiExport for GDYLattice {
    fn output_in_msi(&self) -> String {
        let lattice_vectors = self.get_lattice_vectors();
        let vec_a_line = format!(
            "  (A D A3 ({:.12} {:.12} {:.12}))\n",
            lattice_vectors.column(0).x,
            lattice_vectors.column(0).y,
            lattice_vectors.column(0).z
        );
        let vec_b_line = format!(
            "  (A D B3 ({:.12} {:.12} {:.12}))\n",
            lattice_vectors.column(1).x,
            lattice_vectors.column(1).y,
            lattice_vectors.column(1).z
        );
        let vec_c_line = format!(
            "  (A D C3 ({:.12} {:.12} {:.12}))\n",
            lattice_vectors.column(2).x,
            lattice_vectors.column(2).y,
            lattice_vectors.column(2).z
        );
        let headers: String = vec![
            "# MSI CERIUS2 DataModel File Version 4 0\n",
            "(1 Model\n",
            "  (A I CRY/DISPLAY (192 256))\n",
            "  (A I PeriodicType 100)\n",
            "  (A C SpaceGroup \"1 1\")\n",
            &vec_a_line,
            &vec_b_line,
            &vec_c_line,
            "  (A D CRY/TOLERANCE 0.05)\n",
        ]
        .join("");
        let atom_strings: Vec<String> = self
            .get_atoms()
            .iter()
            .map(|atom| atom.output_in_msi())
            .collect();
        format!("{}{}", headers, atom_strings.concat())
    }
}
