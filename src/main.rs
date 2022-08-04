mod api;

#[macro_use] extern crate rocket;
extern crate env_logger;
use rocket::http::Header;
use rocket::Response;
use rocket::Request;
use rocket::fairing::{ Info, Fairing, Kind };
use rocket::State;
use rocket::response::status;
use rocket::serde::json::Json;
use crate::api::models::{ PumpState, PumpJob, GenericError, settings::Settings };
use crate::api::{ PumpService, SettingsService };

#[options("/pumps")]
fn pumps_options() -> () { }

#[get("/pumps")]
fn pumps_get(pump_service: &State<PumpService>) -> Json<Vec<PumpState>> {
    Json(pump_service.get_pump_states())
}

#[options("/pump_queue")]
fn pump_queue_options() -> () { }

#[get("/pump_queue")]
fn pump_queue_get(pump_service: &State<PumpService>) -> Json<Vec<PumpJob>> {
    Json(pump_service.get_pump_queue())
}

#[options("/pumps/<_pump_number>")]
fn pump_number_options(_pump_number: u8) -> () { }

#[get("/pumps/<pump_number>")]
fn pump_number_get(pump_service: &State<PumpService>, pump_number: u8) -> Result<Json<PumpState>, status::BadRequest::<Json<GenericError>>> {
    match pump_service.get_pump_state(pump_number) {
        Ok(pump_state) => Ok(Json(pump_state)),
        Err(error) => Err(status::BadRequest(Some(Json(GenericError { message: error.to_string() }))))
    }
}

#[post("/pumps/<pump_number>", data = "<ml_to_pump_input>")]
fn pump_number_post(pump_service: &State<PumpService>, pump_number: u8, ml_to_pump_input: String) -> Result<status::Accepted::<Json<Vec<PumpJob>>>, status::BadRequest::<Json<GenericError>>> {
    let temp = ml_to_pump_input.trim();
    if temp.is_empty() {
        return Err(status::BadRequest(Some(Json(GenericError { message: String::from("Expected ml to pump") }))));
    }
    match temp.parse::<u32>() {
        Ok(ml_to_pump) => match pump_service.enqueue_pump(pump_number, ml_to_pump) {
            Ok(pump_queue) => Ok(status::Accepted(Some(Json(pump_queue)))),
            Err(error) => Err(status::BadRequest(Some(Json(GenericError { message: error.to_string() }))))
        },
        Err(_) => Err(status::BadRequest(Some(Json(GenericError { message: String::from("Couldn't parse ml to pump") }))))
    }
}

#[options("/settings")]
fn settings_options() -> () { }

#[get("/settings")]
fn settings_get(settings_service: &State<SettingsService>) -> Json<Settings> {
    Json(settings_service.settings.read().unwrap().clone())
}

#[put("/settings", format = "application/json", data = "<settings_json>")]
fn settings_put(settings_service: &State<SettingsService>, settings_json: Json<Settings>) -> Result<status::NoContent, status::BadRequest::<Json<GenericError>>> {
    let settings = settings_json.into_inner();
    if !settings.is_valid() {
        return Err(status::BadRequest(Some(Json(GenericError { message: String::from("Settings are invalid") }))));
    }
    match settings_service.save(settings) {
        Ok(_) => Ok(status::NoContent),
        Err(save_error) => Err(status::BadRequest(Some(Json(GenericError { message: save_error.to_string() }))))
    }
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PUT, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
    }
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    let pump_service: PumpService;
    match PumpService::new() {
        Ok(new_pump_service) => pump_service = new_pump_service,
        Err(error) => {
            panic!("Couldn't create pump service: {}", error);
        }
    }
    let settings_service: SettingsService;
    match SettingsService::new() {
        Ok(new_settings_service) => settings_service = new_settings_service,
        Err(error) => {
            panic!("Couldn't create settings service: {}", error);
        }
    }
    rocket::build()
        .attach(CORS)
        .mount("/", routes![pumps_options, pumps_get, pump_queue_options, pump_queue_get, pump_number_options, pump_number_get, pump_number_post, settings_options, settings_get, settings_put])
        .manage(pump_service)
        .manage(settings_service)
}
