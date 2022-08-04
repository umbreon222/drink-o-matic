use uuid::Uuid;
use rocket::serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct IngredientMeasurement {
    #[serde(rename  = "ingredientId")]
    pub ingredient_id: Uuid,
    pub parts: u32
}
