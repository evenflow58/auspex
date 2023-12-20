use crate::models::{ability::Ability};
use serde::{Deserialize, Serialize};

use super::enhancment::Enhancement;

// Create a struct named Unit that has the following fields
// Name: String
// Abilities: Ability
// and that's it
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Unit {
    pub name: String,
    pub abilities: Vec<Ability>,
    pub enhancement: Option<Enhancement>,
}

impl Unit {
    pub fn new(name: String,
        abilities: &Vec<Ability>,
        enhancement: Option<Enhancement>,
    ) -> Self {
        Unit {
            name,
            abilities: abilities.to_vec(),
            enhancement,
        }
    }
}