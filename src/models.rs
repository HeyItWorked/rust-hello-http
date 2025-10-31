use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon{
    pub id: u32,
    pub name: String,
    pub poke_type: String,
    pub level: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePokemon{
    pub name: String,
    pub poke_type: String,
    pub level: u32,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePokemon{
    pub name: Option<String>,
    pub poke_type: Option<String>,
    pub level: Option<u32>,
}