use castep_model_generator_backend::{lattice::LatticeModel, model_type::ModelInfo};

extern crate nalgebra;
#[derive(Debug, Clone)]
pub struct GDYLattice<T: ModelInfo> {
    lattice: LatticeModel<T>,
    lattice_name: String,
    metal_site: u32,
    coord_sites: [CoordSite; 7],
}

#[derive(Debug, Clone)]
pub struct CoordSite {
    site_name: String,
    site_id: u32,
}

impl CoordSite {
    pub fn site_name(&self) -> &str {
        self.site_name.as_ref()
    }

    pub fn site_id(&self) -> u32 {
        self.site_id
    }
}

impl<T: ModelInfo> GDYLattice<T> {
    pub fn new(lattice: LatticeModel<T>, lattice_name: String) -> Self {
        let coord_sites: [CoordSite; 7] = [
            CoordSite {
                site_name: "c1".into(),
                site_id: 41,
            },
            CoordSite {
                site_name: "c2".into(),
                site_id: 42,
            },
            CoordSite {
                site_name: "c3".into(),
                site_id: 54,
            },
            CoordSite {
                site_name: "c4".into(),
                site_id: 53,
            },
            CoordSite {
                site_name: "FR".into(),
                site_id: 52,
            },
            CoordSite {
                site_name: "NR".into(),
                site_id: 40,
            },
            CoordSite {
                site_name: "M".into(),
                site_id: 73,
            },
        ];
        Self {
            lattice,
            lattice_name,
            metal_site: 73,
            coord_sites,
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

    pub fn coord_sites(&self) -> &[CoordSite; 7] {
        &self.coord_sites
    }
}

// impl AddAdsorbate for GDYLattice<MsiModel> {
//     fn append_mol_name<Ads: Adsorbate + Clone>(
//         &mut self,
//         ads: &Ads,
//         target_sites: &[u32],
//         coord_site_dict: &HashMap<u32, String>,
//     ) -> Result<(), Box<dyn Error>> {
//         let number_of_sites = target_sites.len();
//         let new_name = match number_of_sites {
//             2 => {
//                 let (site_1, site_2) = (target_sites[0], target_sites[1]);
//                 let suf_1 = coord_site_dict.get(&site_1).unwrap().to_owned();
//                 let suf_2 = coord_site_dict.get(&site_2).unwrap().to_owned();
//                 format!(
//                     "{}_{}_{}_{}",
//                     self.lattice_name(),
//                     ads.get_ads_name(),
//                     suf_1,
//                     suf_2
//                 )
//             }
//             1 => {
//                 let site_1 = target_sites[0];
//                 format!(
//                     "{}_{}_{}",
//                     self.lattice_name(),
//                     ads.get_ads_name(),
//                     coord_site_dict.get(&site_1).unwrap().to_owned()
//                 )
//             }
//             _ => {
//                 panic!("target_sites is empty!");
//             }
//         };
//         self.set_lattice_name(new_name);
//         Ok(())
//     }
//
//     fn init_ads_direction<Ads: Adsorbate + Clone>(
//         &self,
//         ads: &mut Ads,
//         target_sites: &[u32],
//         flip_upright: bool,
//     ) -> Result<(), Box<dyn Error>> {
//         let ads_stem_vec = ads.get_stem_vector()?;
//         let (carbon_chain_a, carbon_chain_b) = match target_sites.len() {
//             2 => (target_sites[0], target_sites[1]),
//             1 => (41, 42),
//             _ => panic!("target_sites is empty!"),
//         };
//         let carbon_chain_direction = self.get_vector_ab(carbon_chain_a, carbon_chain_b)?;
//         let stem_chain_angle = ads_stem_vec.angle(&carbon_chain_direction);
//         let rot_axis = Unit::new_normalize(ads_stem_vec.cross(&carbon_chain_direction));
//         let rot_quatd = UnitQuaternion::from_axis_angle(&rot_axis, stem_chain_angle);
//         ads.rotate(&rot_quatd);
//         if flip_upright == true {
//             ads.make_upright(flip_upright)?;
//         }
//         Ok(())
//     }
//
//     fn add_ads<Ads: Adsorbate + Clone>(
//         &mut self,
//         ads: &mut Ads,
//         target_sites: &[u32],
//         height: f64,
//         flip_upright: bool,
//         coord_site_dict: &HashMap<u32, String>,
//     ) -> Result<(), Box<dyn Error>> {
//         let mut cloned_ads = ads.clone();
//         self.init_ads_direction(&mut cloned_ads, target_sites, flip_upright)?;
//         /*
//         If site_2 exists or the adsorbate has two coordination atoms,
//         the coordinate sites follows the adsorbate info.
//         If site_2 does not exists and the adsorbate has one coordinate atom,
//         both sites are assigned to the only one coordinate atom.
//         */
//         let (cd_1, cd_2) = if target_sites.len() == 2 || cloned_ads.get_coord_atoms().len() == 2 {
//             (
//                 cloned_ads.get_coord_atoms()[0],
//                 cloned_ads.get_coord_atoms()[1],
//             )
//         } else {
//             (
//                 cloned_ads.get_coord_atoms()[0],
//                 cloned_ads.get_coord_atoms()[0],
//             )
//         };
//         let (lat_site_1, lat_site_2) =
//             if target_sites.len() == 2 || cloned_ads.get_coord_atoms().len() == 2 {
//                 (target_sites[0], target_sites[1])
//             } else {
//                 (target_sites[0], target_sites[0])
//             };
//         // Get the center coordinates of the two carbon sites.
//         let lat_sites = (
//             self.get_atom_by_id(lat_site_1)?.xyz().clone(),
//             self.get_atom_by_id(lat_site_2)?.xyz().clone(),
//         );
//         let cd_sites = (
//             cloned_ads.get_atom_by_id(cd_1)?.xyz().clone(),
//             cloned_ads.get_atom_by_id(cd_2)?.xyz().clone(),
//         );
//         let lat_sites_centroid = na::center(&lat_sites.0, &lat_sites.1);
//         let cd_sites_centroid = na::center(&cd_sites.0, &cd_sites.1);
//         let mut trans_matrix = Translation3::from(lat_sites_centroid - cd_sites_centroid);
//         trans_matrix.vector.z += height;
//         cloned_ads.translate(&trans_matrix);
//         let last_id = self.get_atoms().len() as u32;
//         cloned_ads
//             .get_mut_atoms()
//             .iter_mut()
//             .for_each(|atom| atom.set_atom_id(last_id + atom.atom_id()));
//         self.get_mut_atoms().extend(cloned_ads.get_atoms().to_vec());
//         self.append_mol_name(&cloned_ads, target_sites, coord_site_dict)?;
//         Ok(())
//     }
// }
