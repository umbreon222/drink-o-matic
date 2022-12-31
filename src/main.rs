mod api;

use std::sync::{ Mutex, Arc };
#[macro_use] extern crate rocket;
extern crate env_logger;
use rocket::http::Header;
use rocket::Response;
use rocket::Request;
use rocket::fairing::{ Info, Fairing, Kind };
use rocket::State;
use rocket::response::status;
use rocket::serde::json::Json;
use crate::api::models::{ PumpState, PumpJob, GenericError };
use crate::api::models::settings::Settings;
use crate::api::{
    PumpService,
    PumpServiceFactory,
    SettingsService,
    SettingsServiceFactory,
    ResourceService,
    ResourceServiceFactory
};

#[options("/pumps")]
fn pumps_options() -> () { }

#[get("/pumps")]
fn pumps_get(pump_service: &State<Arc<Mutex<PumpService>>>) -> Json<Vec<PumpState>> {
    Json(pump_service.lock().unwrap().get_pump_states())
}

#[options("/pump_queue")]
fn pump_queue_options() -> () { }

#[get("/pump_queue")]
fn pump_queue_get(pump_service: &State<Arc<Mutex<PumpService>>>) -> Json<Vec<PumpJob>> {
    Json(pump_service.lock().unwrap().get_pump_queue())
}

#[options("/pumps/<_pump_number>")]
fn pump_number_options(_pump_number: u8) -> () { }

#[get("/pumps/<pump_number>")]
fn pump_number_get(pump_service: &State<Arc<Mutex<PumpService>>>, pump_number: u8) -> Result<Json<PumpState>, status::BadRequest::<Json<GenericError>>> {
    match pump_service.lock().unwrap().get_pump_state(pump_number) {
        Ok(pump_state) => Ok(Json(pump_state)),
        Err(error) => Err(status::BadRequest(Some(Json(GenericError { message: error.to_string() }))))
    }
}

#[post("/pumps/<pump_number>", data = "<ml_to_pump_input>")]
fn pump_number_post(resource_service: &State<Arc<ResourceService>>, pump_service: &State<Arc<Mutex<PumpService>>>, pump_number: u8, ml_to_pump_input: String) -> Result<status::Accepted::<Json<Vec<PumpJob>>>, status::BadRequest::<Json<GenericError>>> {
    let temp = ml_to_pump_input.trim();
    if temp.is_empty() {
        let expected_ml_to_pump_message = resource_service.get_resource_string_by_name("expected_ml_to_pump_error_message").unwrap();
        return Err(status::BadRequest(Some(Json(GenericError { message: expected_ml_to_pump_message }))));
    }
    match temp.parse::<u32>() {
        Ok(ml_to_pump) => match pump_service.lock().unwrap().enqueue_pump(pump_number, ml_to_pump) {
            Ok(pump_queue) => Ok(status::Accepted(Some(Json(pump_queue)))),
            Err(error) => Err(status::BadRequest(Some(Json(GenericError { message: error.to_string() }))))
        },
        Err(_) => {
            let ml_to_pump_parse_message = resource_service.get_resource_string_by_name("ml_to_pump_parse_error_message").unwrap();
            Err(status::BadRequest(Some(Json(GenericError { message: ml_to_pump_parse_message }))))
        }
    }
}

#[options("/settings")]
fn settings_options() -> () { }

#[get("/settings")]
fn settings_get(settings_service: &State<Arc<SettingsService>>) -> Json<Settings> {
    Json(settings_service.settings.read().unwrap().clone())
}

#[put("/settings", format = "application/json", data = "<settings_json>")]
fn settings_put(resource_service: &State<ResourceService>, settings_service: &State<Arc<SettingsService>>, settings_json: Json<Settings>) -> Result<status::NoContent, status::BadRequest::<Json<GenericError>>> {
    let settings = settings_json.into_inner();
    if !settings.is_valid() {
        let settings_invalid_message = resource_service.get_resource_string_by_name("invalid_settings_error_message").unwrap();
        return Err(status::BadRequest(Some(Json(GenericError { message: settings_invalid_message }))));
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

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Init logger
    env_logger::init();

    // Init dotenv
    let home_dir = dirs::home_dir().unwrap();
    dotenv::from_filename(home_dir.join(".drink-o-matic/.env")).ok();

    // Create resource service
    let resource_service = ResourceServiceFactory::create_or_panic();
    let resource_service_arc = Arc::new(resource_service);

    // Create pump service
    let mut pump_service = PumpServiceFactory::create_or_panic(resource_service_arc.clone());
    let number_of_pumps = pump_service.get_number_of_pumps();
    pump_service.start_daemon();
    let pump_service_arc = Arc::new(Mutex::new(pump_service));

    // Create settings service
    let settings_service = SettingsServiceFactory::create_or_panic(resource_service_arc.clone(), number_of_pumps);
    let settings_service_arc = Arc::new(settings_service);

    let routes = routes![
        pumps_options,
        pumps_get,
        pump_queue_options,
        pump_queue_get,
        pump_number_options,
        pump_number_get,
        pump_number_post,
        settings_options,
        settings_get,
        settings_put
    ];
    
    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", routes)
        .manage(pump_service_arc.clone())
        .manage(settings_service_arc)
        .manage(resource_service_arc)
        .ignite().await?
        .launch().await?;

    pump_service_arc.lock().unwrap().kill_daemon();
    Ok(())
}
