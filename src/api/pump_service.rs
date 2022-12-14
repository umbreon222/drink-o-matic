use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::sync::{ Mutex, Arc, Condvar };
#[cfg(feature = "use-gpio")]
use gpio_cdev::LineHandle;
#[cfg(not(feature = "use-gpio"))]
use crate::api::mock::LineHandle;
use crate::api::models::{ PumpState, PumpJob };

const INVALID_PUMP_NUMBER_ERROR: &str = "Invalid pump number";

pub struct PumpService {
    pub is_relay_inverted: bool,
    pub pump_pin_numbers: Vec<u32>,
    pub ms_per_ml: u64,
    pub daemon_thread: Option<thread::JoinHandle<()>>,
    pub line_handles: Arc<Mutex<Vec<LineHandle>>>,
    pub pump_states: Arc<Mutex<Vec<PumpState>>>,
    pub pump_queue: Arc<Mutex<VecDeque<PumpJob>>>,
    pub run_daemon_pair: Arc<(Mutex<bool>, Condvar)>
}

impl PumpService {
    pub fn get_number_of_pumps(&self) -> u8 {
        self.pump_pin_numbers.len() as u8
    }

    pub fn pump_number_is_valid(pump_number: u8, number_of_pumps: u8) -> bool {
        return pump_number > 0 && pump_number <= number_of_pumps;
    }

    pub fn enqueue_pump(&self, pump_number: u8, ml_to_pump: u32) -> Result<Vec<PumpJob>, &str> {
        if !PumpService::pump_number_is_valid(pump_number, self.get_number_of_pumps()) {
            return Err(INVALID_PUMP_NUMBER_ERROR);
        }
        if ml_to_pump == 0 {
            return Err("ml_to_pump must be greater than 0");
        }
        let duration_in_milliseconds = ml_to_pump as u64 * self.ms_per_ml;
        log::info!("Scheduling pump {} to run for {} ms", pump_number, duration_in_milliseconds);
        self.pump_queue.lock().unwrap().push_back(PumpJob {
            pump_number,
            duration_in_milliseconds
        });
        let pump_queue = self.get_pump_queue();
        self.notify_daemon(false);
        Ok(pump_queue)
    }

    pub fn get_pump_state(&self, pump_number: u8) -> Result<PumpState, &str> {
        if !PumpService::pump_number_is_valid(pump_number, self.get_number_of_pumps()) {
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
        
    pub fn start_daemon(&mut self) {
        if !self.daemon_thread.is_none() {
            return;
        }
        let is_relay_inverted = self.is_relay_inverted.clone();
        let pump_queue_arc = self.pump_queue.clone();
        let line_handles_arc = self.line_handles.clone();
        let pump_states_arc = self.pump_states.clone();
        let run_daemon_pair = self.run_daemon_pair.clone();
        let thread_handle = thread::spawn(move || {
            PumpService::process_queue(is_relay_inverted, pump_queue_arc, line_handles_arc, pump_states_arc, run_daemon_pair);
        });
        self.daemon_thread = Some(thread_handle);
        log::info!("Daemon thread started");
    }
    
    pub fn kill_daemon(&mut self) {
        self.notify_daemon(true);
        if let Some(daemon_thread) = self.daemon_thread.take() {
            daemon_thread.join().unwrap();
            log::info!("Daemon thread killed");
        }
    }
    
    fn notify_daemon(&self, kill_thread: bool) {
        let (lock, cvar) = &*self.run_daemon_pair;
        let mut run_daemon = lock.lock().unwrap();
        *run_daemon = !kill_thread;
        // We notify the condvar that the value has changed.
        cvar.notify_one();
    }

    fn process_queue(
        is_relay_inverted: bool,
        pump_queue_arc: Arc<Mutex<VecDeque<PumpJob>>>,
        line_handles_arc: Arc<Mutex<Vec<LineHandle>>>,
        pump_states_arc: Arc<Mutex<Vec<PumpState>>>,
        should_run_daemon_pair: Arc<(Mutex<bool>, Condvar)>
    ) {
        log::debug!("Starting to pump job queue processor daemon");
        let (should_run_daemon_mutex, cvar) = &*should_run_daemon_pair;
        let mut should_run_daemon = false;
        if let Ok(should_run_daemon_guard) = should_run_daemon_mutex.lock() {
            should_run_daemon = should_run_daemon_guard.clone();
        }
        while should_run_daemon {
            // Get first in line job, leave in queue until done processing
            let mut pump_job_to_process: Option<PumpJob> = None;
            if let Ok(pump_queue) = pump_queue_arc.lock() {
                pump_job_to_process = pump_queue.get(0).copied();
            }
            while let Some(pump_job) = pump_job_to_process {
                let index = pump_job.pump_number as usize - 1;
                let duration: Duration;
                if let Ok(mut locked_pump_states) = pump_states_arc.lock() {
                    log::info!("Processing job to run pump {} for {} ms", pump_job.pump_number, pump_job.duration_in_milliseconds);
                    locked_pump_states[index].is_running = true;
                    duration = Duration::from_millis(pump_job.duration_in_milliseconds);
                }
                else {
                    panic!("Failed to lock pump states");
                }
                if let Ok(locked_line_handles) = line_handles_arc.lock() {
                    let mut high = 1;
                    let mut low = 0;
                    if is_relay_inverted {
                        high = 0;
                        low = 1;
                    }
                    log::debug!("Setting pump {} to HIGH={}", pump_job.pump_number, high);
                    // Force panic if pump value can't be set.
                    locked_line_handles[index].set_value(high).unwrap();
                    thread::sleep(duration);
                    log::debug!("Setting pump {} to LOW={}", pump_job.pump_number, low);
                    locked_line_handles[index].set_value(low).unwrap();
                }
                if let Ok(mut locked_pump_states) = pump_states_arc.lock() {
                    locked_pump_states[pump_job.pump_number as usize - 1].is_running = false;
                }
                if let Ok(mut pump_queue) = pump_queue_arc.lock() {
                    // Discard the job we just processed
                    pump_queue.pop_front();
                    // Get next in line job for processing if any
                    pump_job_to_process = pump_queue.get(0).copied();
                }
                // Intermediate checking for daemon killed
                if let Ok(should_run_daemon_guard) = should_run_daemon_mutex.lock() {
                    if !*should_run_daemon_guard {
                        log::debug!("Pump job queue processor daemon killed while processing jobs");
                        return;
                    }
                }
            }
            log::debug!("Finished processing pump job queue");
            if let Ok(should_run_daemon_guard) = should_run_daemon_mutex.lock() {
                log::debug!("Waiting for \"should run pump queue daemon guard\"");
                let temp_should_run_daemon_guard = cvar.wait(should_run_daemon_guard).unwrap();
                should_run_daemon = temp_should_run_daemon_guard.clone();
                log::debug!("Received \"should run pump queue daemon guard\": {}", should_run_daemon);
            }
        }
        log::debug!("Queue processor daemon killed");
    }
}
