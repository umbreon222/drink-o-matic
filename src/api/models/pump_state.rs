use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct PumpState {
    pub pump_number: u8,
    pub is_running: bool
}
