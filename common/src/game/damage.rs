use crate::game::character::Character;
use crate::game::class::Class;
use crate::game::id_map::IdMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Damage {
    #[serde(default)]
    flat_magical: i32,
    #[serde(default)]
    flat_physical: i32,
    #[serde(default)]
    flat_pure: i32,

    #[serde(default)]
    perc_magical: f32,
    #[serde(default)]
    perc_physical: f32,
    #[serde(default)]
    perc_pure: f32,

    #[serde(default)]
    perc_modif_magical: f32,
    #[serde(default)]
    perc_modif_physical: f32,
}

impl Damage {
    pub fn compute_damage(
        &self,
        classes: &IdMap<Class>,
        attacker: &Character,
        defender: &Character,
    ) -> i32 {
        let c_att = classes.get(attacker.class).expect("Invalid attacker class");
        let c_def = classes.get(defender.class).expect("Invalid attacker class");

        let physical_raw = self.flat_physical
            + ((self.perc_physical * defender.effective_health(c_def) as f32) as i32)
            + ((self.perc_modif_physical * attacker.effective_strength(c_att) as f32) as i32);

        let magical_raw = self.flat_magical
            + ((self.perc_magical * defender.effective_health(c_def) as f32) as i32)
            + ((self.perc_modif_magical * attacker.effective_strength(c_att) as f32) as i32);

        let pure_raw =
            self.flat_pure + ((self.perc_pure * defender.effective_health(c_def) as f32) as i32);

        let physical_final = physical_raw - defender.effective_armor(c_def);
        let magical_final = magical_raw - defender.effective_willpower(c_def);

        physical_final + magical_final + pure_raw
    }
}
