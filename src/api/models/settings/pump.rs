use crate::PumpService;
use uuid::Uuid;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct Pump {
    #[serde(rename  = "pumpNumber")]
    pub pump_number: u8,
    #[serde(rename  = "ingredientId")]
    pub ingredient_id: Option<Uuid>
}

impl Pump {
    pub fn is_valid(&self, number_of_pumps: u8) -> bool {
        return PumpService::pump_number_is_valid(self.pump_number, number_of_pumps);
    }
}
