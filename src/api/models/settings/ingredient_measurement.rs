use uuid::Uuid;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct IngredientMeasurement {
    #[serde(rename  = "ingredientId")]
    pub ingredient_id: Uuid,
    pub parts: u16
}
