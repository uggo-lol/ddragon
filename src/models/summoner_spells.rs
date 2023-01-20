use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::shared::Image;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummonerSpells {
    pub version: String,
    pub data: HashMap<String, SummonerSpell>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummonerSpell {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tooltip: String,
    pub maxrank: i64,
    pub cooldown: Vec<i64>,
    #[serde(rename = "cooldownBurn")]
    pub cooldown_burn: String,
    pub cost: Vec<i64>,
    #[serde(rename = "costBurn")]
    pub cost_burn: String,
    pub effect: Vec<Option<Vec<f64>>>,
    #[serde(rename = "effectBurn")]
    pub effect_burn: Vec<Option<String>>,
    pub key: String,
    #[serde(rename = "summonerLevel")]
    pub summoner_level: i64,
    pub modes: Vec<String>,
    #[serde(rename = "costType")]
    pub cost_type: CostType,
    pub maxammo: String,
    pub range: Vec<i64>,
    #[serde(rename = "rangeBurn")]
    pub range_burn: String,
    pub image: Image<SummonerSpellSprite>,
    pub resource: CostType,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum CostType {
    #[serde(rename = "&nbsp;")]
    Nbsp,
    #[serde(rename = "No Cost")]
    NoCost,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Group {
    #[serde(rename = "spell")]
    Spell,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum SummonerSpellSprite {
    #[serde(rename = "spell0.png")]
    Spell0Png,
}
