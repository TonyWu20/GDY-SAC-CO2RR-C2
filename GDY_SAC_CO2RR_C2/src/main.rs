#![allow(non_snake_case)]

use std::error::Error;

use basic_models::gdy_model_edit::generate_all_metal_models;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    generate_all_metal_models()?;
    Ok(())
}
