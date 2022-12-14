use std::collections::VecDeque;
use std::sync::{ Arc, Mutex, Condvar };
#[cfg(feature = "use-gpio")]
use gpio_cdev::{ Chip, LineRequestFlags, LineHandle };
#[cfg(not(feature = "use-gpio"))]
use crate::api::mock::{ Chip, LineRequestFlags, LineHandle };
use crate::api::models::PumpState;
use crate::PumpService;

pub struct PumpServiceFactory {}

impl PumpServiceFactory {
    pub fn create_or_panic() -> PumpService {
        let is_relay_inverted = dotenv::var("IS_RELAY_INVERTED").unwrap().ends_with('1');
        let ms_per_ml = dotenv::var("MILLISECONDS_PER_ML").unwrap().parse::<u64>().unwrap();
        let rpi_chip_name = dotenv::var("RPI_CHIP_NAME").unwrap();
        let pump_pin_numbers_string = dotenv::var("ORDERED_PUMP_PIN_NUMBERS").unwrap();
        let pump_pin_numbers: Vec<u32> = pump_pin_numbers_string.split(',').map(|num| num.parse::<u32>().unwrap()).collect();
        let line_handles = Self::get_line_handles(rpi_chip_name, &pump_pin_numbers, is_relay_inverted);
        let initial_pump_states = (1..=pump_pin_numbers.len() as u8).map(|pump_number| PumpState { pump_number, is_running: is_relay_inverted }).collect();

        PumpService {
            is_relay_inverted,
            ms_per_ml,
            pump_pin_numbers,
            daemon_thread: None,
            line_handles: Arc::new(Mutex::new(line_handles)), // Revise all 3 of these with RwLock where appropriate
            pump_states: Arc::new(Mutex::new(initial_pump_states)),
            pump_queue: Arc::new(Mutex::new(VecDeque::new())),
            run_daemon_pair: Arc::new((Mutex::new(true), Condvar::new()))
        }
    }

    fn get_line_handles(rpi_chip_name: String, pump_pin_numbers: &Vec<u32>, is_relay_inverted: bool) -> Vec<LineHandle> {
        if cfg!(not(feature = "use-gpio")) {
            log::info!("Feature \"use-gpio\" was not set; GPIO will be mocked");
        }

        log::info!("Getting chip \"{}\"", rpi_chip_name);
        let mut chip = Chip::new(&rpi_chip_name).unwrap();
        let mut default_state: u8 = 0;
        if is_relay_inverted {
            default_state = 1;
        }

        let mut line_handles: Vec<LineHandle> = vec![];
        let mut pump_number = 1;
        for pin_number in pump_pin_numbers {
            let line = chip.get_line(*pin_number).unwrap();
            line_handles.push(line.request(LineRequestFlags::OUTPUT, default_state, format!("Pump {}", pump_number).as_str()).unwrap());
            pump_number += 1;
        }

        line_handles
    }
}
