use crate::Pathway;

#[derive(Debug)]
/// Marker struct to represent "CH2 coupling pathway"
pub struct CH2Pathway;
impl Pathway for CH2Pathway {}

#[derive(Debug)]
/// Marker struct to represent "CO dimerization pathway"
pub struct COPathway;
impl Pathway for COPathway {}
