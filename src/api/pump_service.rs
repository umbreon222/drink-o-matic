use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::sync::{ Mutex, Arc };
use core::sync::atomic::{ AtomicBool, Ordering };
#[cfg(feature = "use-gpio")]
use gpio_cdev::{ Chip, Line, LineRequestFlags, LineHandle };
#[cfg(not(feature = "use-gpio"))]
use crate::mock::{ Chip, Line, LineRequestFlags, LineHandle };
use crate::api::models::{ PumpState, PumpJob };

const INVALID_PUMP_NUMBER_ERROR: &str = "Invalid pump number";

const RPI_CHIP_NAME: &str = "/dev/gpiochip0";
const NUMBER_OF_PUMPS: usize = 8;
const PUMP_PIN_NUMBERS: [u32; NUMBER_OF_PUMPS] = [
    1, // PUMP 1
    2, // PUMP 2
    3, // PUMP 3
    4, // PUMP 4
    5, // PUMP 5
    6, // PUMP 6
    7, // PUMP 7
    8, // PUMP 8
];
const MILLISECONDS_PER_ML: u32 = 1000;

static IS_PROCESSING_QUEUE: AtomicBool = AtomicBool::new(false);

pub struct PumpService {
    line_handles: Arc<Mutex<Vec<LineHandle>>>,
    pump_states: Arc<Mutex<Vec<PumpState>>>,
    pump_queue: Arc<Mutex<VecDeque<PumpJob>>>
}

impl PumpService {
    pub fn new() -> Result<Self, String> {
        let mut chip: Chip;
        log::info!("Getting chip \"{}\"", RPI_CHIP_NAME);
        match Chip::new(RPI_CHIP_NAME) {
            Ok(res) => chip = res,
            Err(e) => return Err(format!("Error getting chip \"{}\": {}", RPI_CHIP_NAME, e))
        }
        let mut line_handles: Vec<LineHandle> = vec![];
        let mut pump_states: Vec<PumpState> = vec![];
        for pump_index in 0..NUMBER_OF_PUMPS {
            let line: Line;
            let pin_number = PUMP_PIN_NUMBERS[pump_index];
            let pump_number = pump_index as u8 + 1;
            log::info!("Getting line handle for pump {} on pin {}", pump_number, pin_number);
            match chip.get_line(pin_number) {
                Ok(res) => line = res,
                Err(e) => return Err(format!("Error getting line for pump {} on pin {}: {}", pump_number, pin_number, e))
            }
            match line.request(LineRequestFlags::OUTPUT, 0, format!("Pump {}", pump_number).as_str()) {
                Ok(line_handle) => line_handles.push(line_handle),
                Err(e) => return Err(format!("Error getting line handle for pump {} on pin {}: {}", pump_number, pin_number, e))
            }
            pump_states.push(PumpState {
                pump_number,
                is_running: false
            });
        }
        Ok(Self {
            line_handles: Arc::new(Mutex::new(line_handles)),
            pump_states: Arc::new(Mutex::new(pump_states)),
            pump_queue: Arc::new(Mutex::new(VecDeque::new()))
        })
    }

    pub fn enqueue_pump(&self, pump_number: u8, ml_to_pump: u8) -> Result<Vec<PumpJob>, &str> {
        if pump_number == 0 || pump_number > NUMBER_OF_PUMPS as u8 {
            return Err(INVALID_PUMP_NUMBER_ERROR);
        }
        if ml_to_pump == 0 {
            return Err("ml_to_pump must be greater than 0");
        }
        let duration_in_milliseconds = ml_to_pump as u32 * MILLISECONDS_PER_ML;
        log::info!("Scheduling pump {} to run for {} ms", pump_number, duration_in_milliseconds);
        self.pump_queue.lock().unwrap().push_back(PumpJob {
            pump_number: pump_number,
            duration_in_milliseconds
        });
        let result = IS_PROCESSING_QUEUE.compare_exchange(false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed);
        if result.is_ok() {
            let pump_queue_arc = self.pump_queue.clone();
            let line_handles_arc = self.line_handles.clone();
            let pump_states_arc = self.pump_states.clone();
            let _ = thread::spawn(move || {
                PumpService::process_queue(pump_queue_arc, line_handles_arc, pump_states_arc);
                IS_PROCESSING_QUEUE.store(false, Ordering::Release);
            });
        }
        Ok(self.get_pump_queue())
    }

    pub fn get_pump_state(&self, pump_number: u8) -> Result<PumpState, &str> {
        if pump_number == 0 || pump_number > NUMBER_OF_PUMPS as u8 {
            return Err(INVALID_PUMP_NUMBER_ERROR);
        }
        Ok(self.pump_states.lock().unwrap()[pump_number as usize - 1].clone())
    }

    pub fn get_pump_states(&self) -> Vec<PumpState> {
        self.pump_states.lock().unwrap().clone()
    }

    pub fn get_pump_queue(&self) -> Vec<PumpJob> {
        Vec::from(self.pump_queue.lock().unwrap().clone())
    }

    pub fn process_queue(
        pump_queue_arc: Arc<Mutex<VecDeque<PumpJob>>>,
        line_handles_arc: Arc<Mutex<Vec<LineHandle>>>,
        pump_states_arc: Arc<Mutex<Vec<PumpState>>>
    ) {
        log::info!("Starting to process queue");
        let mut first_in_queue = pump_queue_arc.lock().unwrap().pop_front();
        while first_in_queue.is_some() {
            let pump_job = first_in_queue.unwrap();
            let index = pump_job.pump_number as usize - 1;
            // Add the first element back so the user can get the queue state accurately.
            pump_queue_arc.lock().unwrap().push_front(pump_job.clone());
            let duration: Duration;
            if let Ok(mut locked_pump_states) = pump_states_arc.lock() {
                log::info!("Processing job to run pump {} for {} ms", pump_job.pump_number, pump_job.duration_in_milliseconds);
                locked_pump_states[index].is_running = true;
                duration = Duration::from_millis(pump_job.duration_in_milliseconds as u64);
            }
            else {
                panic!("Failed to lock pump states");
            }
            if let Ok(locked_line_handles) = line_handles_arc.lock() {
                log::info!("Setting pump {} to HIGH", pump_job.pump_number);
                // Force panic if pump value can't be set.
                locked_line_handles[index].set_value(1).unwrap();
                thread::sleep(duration);
                log::info!("Setting pump {} to LOW", pump_job.pump_number);
                locked_line_handles[index].set_value(0).unwrap();
            }
            if let Ok(mut locked_pump_states) = pump_states_arc.lock() {
                locked_pump_states[pump_job.pump_number as usize - 1].is_running = false;
            }
            pump_queue_arc.lock().unwrap().pop_front(); // Discard the element we just processed
            first_in_queue = pump_queue_arc.lock().unwrap().pop_front();
        }
        log::info!("Finished processing queue");
    }
}