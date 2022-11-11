#![allow(dead_code)]
pub mod gdy_lattice;
pub mod gdy_model_edit;

extern crate castep_periodic_table as cpt;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
