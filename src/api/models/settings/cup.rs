use uuid::Uuid;
use rocket::serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Cup {
    pub id: Uuid,
    pub name: String,
    #[serde(rename  = "volumeMl")]
    pub volume_ml: u32
}
