use crate::api::models::settings::{ Ingredient, Pump, Drink, Cup };
use rocket::serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Settings {
    pub cups: Vec<Cup>,
    pub ingredients: Vec<Ingredient>,
    pub pumps: Vec<Pump>,
    pub drinks: Vec<Drink>
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            cups: vec![],
            ingredients: vec![],
            pumps: vec![],
            drinks: vec![]
        }
    }

    pub fn is_valid(&self) -> bool {
        // Evaluate relationships between entities
        let mut all_ids = vec![];
        let mut pump_numbers = vec![];
        // Check that all cups are unique
        for cup in &self.cups {
            if all_ids.contains(&cup.id) {
                return false;
            }
            all_ids.push(cup.id);
        }
        // Check that all ingredients are unique
        for ingredient in &self.ingredients {
            if all_ids.contains(&ingredient.id) {
                return false;
            }
            all_ids.push(ingredient.id.clone());
        }
        // Check that all pumps are valid, unique, have a valid ingredient
        for pump in &self.pumps {
            if !pump.is_valid() || pump_numbers.contains(&pump.pump_number) {
                return false;
            }
            pump_numbers.push(pump.pump_number);
            // Pumps can have the same ingredient as each other
            if pump.ingredient_id.is_some() && !all_ids.contains(&pump.ingredient_id.unwrap()) {
                return false;
            }
        }
        // Check that all drinks are valid, unique, have valid ingredient ids
        for drink in &self.drinks {
            if !drink.is_valid() || all_ids.contains(&drink.id) {
                return false;
            }
            all_ids.push(drink.id.clone());
            for ingredient_measurement in &drink.ingredient_measurements {
                if !all_ids.contains(&ingredient_measurement.ingredient_id) {
                    return false;
                }
            }
        }
        return true;
    }
}
