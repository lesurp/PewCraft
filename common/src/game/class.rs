use serde::{Deserialize, Serialize};

// NOTE: we could generate the "Class" struct and the "Attribute" enum jonitly with e.g. a macro
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Attribute {
    Health,
    Swiftness,
    Strength,
    Dexterity,
    Armor,
    Intelligence,
    Concentration,
    Willpower,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Class {
    pub name: String,
    pub description: String,

    // GENERIC STUFF
    pub health: i32,
    pub mana: i32,
    pub swiftness: i32, // movement units

    // PHYSICAL STUFF
    pub strength: i32,  // dmg
    pub dexterity: i32, // precision
    pub armor: i32,     // defense

    // MAGICAL STUFF
    pub intelligence: i32,  // dmg
    pub concentration: i32, // mana (precision for spell is always 100%)
    pub willpower: i32,     // spell defense
}
