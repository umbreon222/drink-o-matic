use rocket::serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct PumpJob {
    pub pump_number: u8,
    pub duration_in_milliseconds: u32
}