use crate::api::models::settings::IngredientMeasurement;
use uuid::Uuid;
use rocket::serde::{ Deserialize, Serialize };

const STAR_RATING_MIN: u8 = 0;
const STAR_RATING_MAX: u8 = 5;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Drink {
    pub id: Uuid,
    #[serde(rename  = "imageUrl")]
    pub image_url: String,
    pub name: String,
    pub description: String,
    #[serde(rename  = "ingredientMeasurements")]
    pub ingredient_measurements: Vec<IngredientMeasurement>,
    #[serde(rename  = "defaultCupId")]
    pub default_cup_id: Option<Uuid>,
    #[serde(rename  = "starRating")]
    pub star_rating: u8
}

impl Drink {
    pub fn is_valid(&self) -> bool {
        return self.star_rating >= STAR_RATING_MIN && self.star_rating <= STAR_RATING_MAX;
    }
}
