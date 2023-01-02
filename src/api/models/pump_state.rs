use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct PumpState {
    #[serde(rename = "pumpNumber")]
    pub pump_number: u8,
    #[serde(rename = "isRunning")]
    pub is_running: bool
}
