use rocket::serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct PumpState {
    pub pump_number: u8,
    pub is_running: bool,
    pub duration_in_milliseconds: u32
}