use uuid::Uuid;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct Cup {
    pub id: Uuid,
    #[serde(rename  = "imageUrl")]
    pub image_url: String,
    pub name: String,
    #[serde(rename  = "volumeMl")]
    pub volume_ml: u16
}
