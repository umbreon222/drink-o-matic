use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct PumpJob {
    pub pump_number: u8,
    pub duration_in_milliseconds: u64
}
