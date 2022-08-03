use crate::PumpService;
use uuid::Uuid;
use rocket::serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Pump {
    #[serde(alias = "pumpNumber")]
    pub pump_number: u8,
    #[serde(alias = "ingredientId")]
    pub ingredient_id: Option<Uuid>
}

impl Pump {
    pub fn is_valid(&self) -> bool {
        return PumpService::pump_number_is_valid(self.pump_number);
    }
}