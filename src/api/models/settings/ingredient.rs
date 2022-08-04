use uuid::Uuid;
use rocket::serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Ingredient {
    pub id: Uuid,
    pub name: String,
    pub modifier: u8,
}