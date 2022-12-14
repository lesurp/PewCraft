use crate::game::class::Attribute;
use crate::game::damage::Damage;
use crate::game::id_map::Id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Anything, // can target empty cells e.g. for AoE
    Anyone,   // can target either allies or enemies
    Enemy,    // can only target enemies
    Ally,     // can only target allies
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RangeKind {
    // range is used as a L1 norm e.g. 1 means the four direct squares around the target are
    // affected as well
    Star,

    // straight line, in the four main directions
    Cross,

    // range is used for each x and y axis e.g. 1 means the 8 squared around the target are in the
    // range
    Square,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Range {
    // number of cells from initial one e.g. 0 means only on oneself
    pub min: u32,
    pub max: u32,

    // how should the area be covered
    pub kind: RangeKind,

    pub target: Target,

    // does this range needs line of sight (e.g. works through walls...)
    pub needs_los: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ValueKind {
    Perc(f32),
    Flat(i32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BuffKind {
    // negative values mean healing i.e. HoT
    DoT(Damage),
    // TODO change that
    Attribute(Attribute, ValueKind),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Buff {
    name: String,
    /// None means the damage will be applied to the given cell only (i.e. no "explosion" around
    /// the target)
    range: Option<Range>,
    success_rate: Option<f32>,
    duration: i32,
    kind: BuffKind,
    // TODO: add initial stats of attacker somewhere here
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectDamage {
    /// None means the damage will be applied to the given cell only (i.e. no "explosion" around
    /// the target)
    pub range: Option<Range>,
    pub damage: Damage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EffectKind {
    DirectDamage(DirectDamage),
    Buff(Buff),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Effect {
    pub id: Id<Effect>,
    pub kind: EffectKind,
}
