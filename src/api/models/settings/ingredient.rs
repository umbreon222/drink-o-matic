use uuid::Uuid;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct Ingredient {
    pub id: Uuid,
    pub name: String,
    pub modifier: u16,
}