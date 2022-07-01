mod api;

#[macro_use] extern crate rocket;
extern crate env_logger;
use rocket::State;
use rocket::response::status;
use rocket::serde::json::Json;
use crate::api::models::{ PumpState, PumpJob, InputError };
use crate::api::PumpService;

#[get("/pumps")]
fn pumps_get(pump_service: &State<PumpService>) -> Json<Vec<PumpState>> {
    Json(pump_service.get_pump_states())
}

#[get("/pump_queue")]
fn pump_queue_get(pump_service: &State<PumpService>) -> Json<Vec<PumpJob>> {
    Json(pump_service.get_pump_queue())
}

#[get("/pumps/<pump_number>")]
fn pump_number_get(pump_service: &State<PumpService>, pump_number: u8) -> Result<Json<PumpState>, status::BadRequest::<Json<InputError>>> {
    match pump_service.get_pump_state(pump_number) {
        Ok(pump_state) => Ok(Json(pump_state)),
        Err(error) => Err(status::BadRequest(Some(Json(InputError { message: error.to_string() }))))
    }
}

#[post("/pumps/<pump_number>", data = "<ml_to_pump_input>")]
fn pump_number_post(pump_service: &State<PumpService>, pump_number: u8, ml_to_pump_input: String) -> Result<Json<Vec<PumpJob>>, status::BadRequest::<Json<InputError>>> {
    let temp = ml_to_pump_input.trim();
    if temp.is_empty() {
        return Err(status::BadRequest(Some(Json(InputError { message: String::from("Expected ml to pump") }))));
    }
    match temp.parse::<u8>() {
        Ok(ml_to_pump) => match pump_service.enqueue_pump(pump_number, ml_to_pump) {
            Ok(pump_queue) => Ok(Json(pump_queue)),
            Err(error) => Err(status::BadRequest(Some(Json(InputError { message: error.to_string() }))))
        },
        Err(_) => Err(status::BadRequest(Some(Json(InputError { message: String::from("Couldn't parse ml to pump") }))))
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
    rocket::build()
        .mount("/", routes![pumps_get, pump_queue_get, pump_number_get, pump_number_post])
        .manage(pump_service)
}