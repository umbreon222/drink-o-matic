use serde::Serialize;

#[derive(Serialize)]
pub struct GenericError {
    pub message: String
}
