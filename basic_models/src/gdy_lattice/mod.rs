use castep_model_core::{CellModel, LatticeModel, ModelInfo, MsiModel};

extern crate nalgebra;
#[derive(Debug, Clone)]
pub struct GDYLattice<T: ModelInfo> {
    lattice: LatticeModel<T>,
    lattice_name: String,
    metal_site: u32,
}

#[derive(Debug, Clone)]
pub struct CoordSite<'a> {
    site_name: &'a str,
    site_id: u32,
}

pub const GDY_SINGLE_CASES: [u32; 7] = [41, 42, 54, 53, 52, 40, 73];
pub const GDY_DOUBLE_CASES: [[u32; 2]; 7] = [
    [41, 42],
    [42, 54],
    [54, 53],
    [53, 52],
    [41, 40],
    [41, 73],
    [42, 73],
];
pub const GDY_COORD_SITES: [CoordSite; 7] = [
    CoordSite {
        site_name: "c1",
        site_id: 41,
    },
    CoordSite {
        site_name: "c2",
        site_id: 42,
    },
    CoordSite {
        site_name: "c3",
        site_id: 54,
    },
    CoordSite {
        site_name: "c4",
        site_id: 53,
    },
    CoordSite {
        site_name: "FR",
        site_id: 52,
    },
    CoordSite {
        site_name: "NR",
        site_id: 40,
    },
    CoordSite {
        site_name: "M",
        site_id: 73,
    },
];

impl<'a> CoordSite<'a> {
    pub fn site_name(&self) -> &str {
        self.site_name
    }
    pub fn site_id(&self) -> u32 {
        self.site_id
    }
}

impl<T: ModelInfo> GDYLattice<T> {
    pub fn new(lattice: LatticeModel<T>, lattice_name: String) -> Self {
        Self {
            lattice,
            lattice_name,
            metal_site: 73,
        }
    }

    pub fn lattice(&self) -> &LatticeModel<T> {
        &self.lattice
    }

    pub fn lattice_name(&self) -> &str {
        self.lattice_name.as_ref()
    }

    pub fn metal_site(&self) -> u32 {
        self.metal_site
    }

    pub fn set_lattice_name(&mut self, lattice_name: String) {
        self.lattice_name = lattice_name;
    }

    pub fn lattice_mut(&mut self) -> &mut LatticeModel<T> {
        &mut self.lattice
    }
}

impl From<GDYLattice<CellModel>> for GDYLattice<MsiModel> {
    fn from(cell: GDYLattice<CellModel>) -> Self {
        Self {
            lattice: cell.lattice.into(),
            lattice_name: cell.lattice_name,
            metal_site: cell.metal_site,
        }
    }
}

impl From<GDYLattice<MsiModel>> for GDYLattice<CellModel> {
    fn from(msi: GDYLattice<MsiModel>) -> Self {
        Self {
            lattice: msi.lattice.into(),
            lattice_name: msi.lattice_name,
            metal_site: msi.metal_site,
        }
    }
}
