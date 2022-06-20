use crate::api::models::PumpJob;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::sync::{ Mutex, Arc };
use core::sync::atomic::{ AtomicBool, Ordering };
use wiringpi::pin;
use crate::api::models::PumpState;

const INVALID_PUMP_NUMBER_ERROR: &str = "Invalid pump number";

const NUMBER_OF_PUMPS: u8 = 8;
const PUMP_1_PIN: u16 = 0;
const PUMP_2_PIN: u16 = 1;
const PUMP_3_PIN: u16 = 2;
const PUMP_4_PIN: u16 = 3;
const PUMP_5_PIN: u16 = 4;
const PUMP_6_PIN: u16 = 5;
const PUMP_7_PIN: u16 = 6;
const PUMP_8_PIN: u16 = 7;
const MILLISECONDS_PER_ML: u32 = 1000;

static IS_PROCESSING_QUEUE: AtomicBool = AtomicBool::new(false);

pub struct PumpService {
    pump_pins: Arc<Mutex<[pin::OutputPin<pin::WiringPi>; NUMBER_OF_PUMPS as usize]>>,
    pump_states: Arc<Mutex<Vec<PumpState>>>,
    pump_queue: Arc<Mutex<VecDeque<PumpJob>>>
}

// TODO: This shit is hella rough so we should clean it up at some point.
impl PumpService {
    pub fn new() -> Self {
        let pi = wiringpi::setup();
        let pump_pins = Arc::new(Mutex::new([
            pi.output_pin(PUMP_1_PIN),
            pi.output_pin(PUMP_2_PIN),
            pi.output_pin(PUMP_3_PIN),
            pi.output_pin(PUMP_4_PIN),
            pi.output_pin(PUMP_5_PIN),
            pi.output_pin(PUMP_6_PIN),
            pi.output_pin(PUMP_7_PIN),
            pi.output_pin(PUMP_8_PIN),
        ]));
        let mut pump_states = vec![];
        for pump_number in 1..=NUMBER_OF_PUMPS {
            pump_states.push(PumpState {
                pump_number: pump_number as u8,
                is_running: false
            });
        }
        Self {
            pump_pins: pump_pins,
            pump_states: Arc::new(Mutex::new(pump_states)),
            pump_queue: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    pub fn enqueue_pump(&self, pump_number: u8, ml_to_pump: u8) -> Result<Vec<PumpJob>, &str> {
        if pump_number == 0 || pump_number > NUMBER_OF_PUMPS {
            return Err(INVALID_PUMP_NUMBER_ERROR);
        }
        if ml_to_pump == 0 {
            return Err("ml_to_pump must be greater than 0");
        }
        self.pump_queue.lock().unwrap().push_back(PumpJob {
            pump_number: pump_number,
            duration_in_milliseconds: ml_to_pump as u32 * MILLISECONDS_PER_ML
        });
        let result = IS_PROCESSING_QUEUE.compare_exchange(false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed);
        if result.is_ok() {
            let pump_queue_arc = self.pump_queue.clone();
            let pump_pins_arc = self.pump_pins.clone();
            let pump_states_arc = self.pump_states.clone();
            let _ = thread::spawn(move || {
                PumpService::process_queue(pump_queue_arc, pump_pins_arc, pump_states_arc);
                IS_PROCESSING_QUEUE.store(false, Ordering::Release);
            });
        }
        Ok(self.get_pump_queue())
    }

    pub fn get_pump_state(&self, pump_number: u8) -> Result<PumpState, &str> {
        if pump_number == 0 || pump_number > NUMBER_OF_PUMPS {
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
        pump_pins_arc: Arc<Mutex<[pin::OutputPin<pin::WiringPi>; NUMBER_OF_PUMPS as usize]>>,
        pump_states_arc: Arc<Mutex<Vec<PumpState>>>
    ) {
        let mut first_in_queue = pump_queue_arc.lock().unwrap().pop_front();
        while first_in_queue.is_some() {
            let pump_job = first_in_queue.unwrap();
            // Add the first element back so the user can get the queue state accurately.
            pump_queue_arc.lock().unwrap().push_front(pump_job.clone());
            let duration: Duration;
            if let Ok(mut locked_pump_states) = pump_states_arc.lock() {
                locked_pump_states[pump_job.pump_number as usize - 1].is_running = true;
                duration = Duration::from_millis(pump_job.duration_in_milliseconds as u64);
            } else {
                panic!("Failed to lock pump states");
            }
            if let Ok(locked_pump_pins) = pump_pins_arc.lock() {
                locked_pump_pins[pump_job.pump_number as usize - 1].digital_write(pin::Value::High);
                thread::sleep(duration);
                locked_pump_pins[pump_job.pump_number as usize - 1].digital_write(pin::Value::Low);
            }
            if let Ok(mut locked_pump_states) = pump_states_arc.lock() {
                locked_pump_states[pump_job.pump_number as usize - 1].is_running = false;
            }
            pump_queue_arc.lock().unwrap().pop_front(); // Discard the element we just processed
            first_in_queue = pump_queue_arc.lock().unwrap().pop_front();
        }
    }
}