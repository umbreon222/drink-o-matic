use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::sync::{ Mutex, Arc, Condvar };
#[cfg(feature = "use-gpio")]
use gpio_cdev::LineHandle;
use serde_json::json;
#[cfg(not(feature = "use-gpio"))]
use crate::api::mock::LineHandle;
use crate::api::models::{ PumpState, PumpJob };
use crate::api::ResourceService;

const INVALID_PUMP_NUMBER_ERROR: &str = "Invalid pump number";

pub struct PumpService {
    resource_service: ResourceService,
    is_relay_inverted: bool,
    pump_pin_numbers: Vec<u32>,
    ms_per_ml: u64,
    daemon_thread: Option<thread::JoinHandle<()>>,
    line_handles: Arc<Mutex<Vec<LineHandle>>>,
    pump_states: Arc<Mutex<Vec<PumpState>>>,
    pump_queue: Arc<Mutex<VecDeque<PumpJob>>>,
    run_daemon_pair: Arc<(Mutex<bool>, Condvar)>
}

impl PumpService {
    pub fn new(
        resource_service: ResourceService,
        is_relay_inverted: bool,
        pump_pin_numbers: Vec<u32>,
        ms_per_ml: u64,
        daemon_thread: Option<thread::JoinHandle<()>>,
        line_handles: Arc<Mutex<Vec<LineHandle>>>,
        pump_states: Arc<Mutex<Vec<PumpState>>>,
        pump_queue: Arc<Mutex<VecDeque<PumpJob>>>,
        run_daemon_pair: Arc<(Mutex<bool>, Condvar)>
    ) -> PumpService {
        PumpService {
            resource_service,
            is_relay_inverted,
            pump_pin_numbers,
            ms_per_ml,
            daemon_thread,
            line_handles,
            pump_states,
            pump_queue,
            run_daemon_pair
        }
    }

    pub fn get_number_of_pumps(&self) -> u8 {
        self.pump_pin_numbers.len() as u8
    }

    pub fn pump_number_is_valid(pump_number: u8, number_of_pumps: u8) -> bool {
        return pump_number > 0 && pump_number <= number_of_pumps;
    }
    
    pub fn enqueue_pump(&self, pump_number: u8, ml_to_pump: u32) -> Result<Vec<PumpJob>, String> {
        if !PumpService::pump_number_is_valid(pump_number, self.get_number_of_pumps()) {
            return Err(INVALID_PUMP_NUMBER_ERROR.to_string());
        }
        if ml_to_pump == 0 {
            let invalid_ml_to_pump_message = self.resource_service.get_resource_string_by_name("invalid_ml_to_pump_error_message").unwrap();
            return Err(invalid_ml_to_pump_message);
        }
        let duration_in_milliseconds = ml_to_pump as u64 * self.ms_per_ml;
        let message_data = &json!({"pump_number": pump_number, "milliseconds": duration_in_milliseconds});
        let scheduling_pump_message = self.resource_service.render_resource_template_string_by_name("scheduling_pump_info_message_template", message_data).unwrap();
        log::info!("{}", scheduling_pump_message);
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
        let resource_service = self.resource_service.clone();
        let is_relay_inverted = self.is_relay_inverted.clone();
        let pump_queue_arc = self.pump_queue.clone();
        let line_handles_arc = self.line_handles.clone();
        let pump_states_arc = self.pump_states.clone();
        let run_daemon_pair = self.run_daemon_pair.clone();
        let thread_handle = thread::spawn(move || {
            PumpService::process_queue(
                resource_service,
                is_relay_inverted, pump_queue_arc,
                line_handles_arc, pump_states_arc,
                run_daemon_pair
            );
        });
        self.daemon_thread = Some(thread_handle);
        let started_daemon_thread_message = self.resource_service.get_resource_string_by_name("daemon_thread_started_message").unwrap();
        log::info!("{}", started_daemon_thread_message);
    }
    
    pub fn kill_daemon(&mut self) {
        self.notify_daemon(true);
        if let Some(daemon_thread) = self.daemon_thread.take() {
            daemon_thread.join().unwrap();
            let killed_daemon_thread_message = self.resource_service.get_resource_string_by_name("daemon_thread_killed_message").unwrap();
            log::info!("{}", killed_daemon_thread_message);
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
        resource_service: ResourceService,
        is_relay_inverted: bool,
        pump_queue_arc: Arc<Mutex<VecDeque<PumpJob>>>,
        line_handles_arc: Arc<Mutex<Vec<LineHandle>>>,
        pump_states_arc: Arc<Mutex<Vec<PumpState>>>,
        should_run_daemon_pair: Arc<(Mutex<bool>, Condvar)>
    ) {
        let starting_daemon_thread_message = resource_service.get_resource_string_by_name("starting_daemon_thread_message").unwrap();
        log::debug!("{}", starting_daemon_thread_message);
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
                    let processing_job_message_data = &json!({"pump_number": pump_job.pump_number, "milliseconds": pump_job.duration_in_milliseconds});
                    let processing_job_message = resource_service.render_resource_template_string_by_name("processing_job_info_message_template", processing_job_message_data).unwrap();
                    log::info!("{}", processing_job_message);
                    locked_pump_states[index].is_running = true;
                    duration = Duration::from_millis(pump_job.duration_in_milliseconds);
                }
                else {
                    let failed_to_lock_pump_states_error_message = resource_service.get_resource_string_by_name("failed_to_lock_pump_states_error_message").unwrap();
                    panic!("{}", failed_to_lock_pump_states_error_message);
                }
                if let Ok(locked_line_handles) = line_handles_arc.lock() {
                    let mut high = 1;
                    let mut low = 0;
                    if is_relay_inverted {
                        high = 0;
                        low = 1;
                    }
                    let setting_pump_high_message_data = &json!({ "pump_number": pump_job.pump_number, "value": high });
                    let setting_pump_high_message = resource_service.render_resource_template_string_by_name("setting_pump_high_info_message_template", setting_pump_high_message_data).unwrap();
                    log::debug!("{}", setting_pump_high_message);
                    locked_line_handles[index].set_value(high).unwrap();
                    thread::sleep(duration);
                    let setting_pump_low_message_data = &json!({ "pump_number": pump_job.pump_number, "value": low });
                    let setting_pump_low_message = resource_service.render_resource_template_string_by_name("setting_pump_low_info_message_template", setting_pump_low_message_data).unwrap();
                    log::debug!("{}", setting_pump_low_message);
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
                        let daemon_killed_while_processing_message = resource_service.get_resource_string_by_name("daemon_killed_while_processing_message").unwrap();
                        log::debug!("{}", daemon_killed_while_processing_message);
                        return;
                    }
                }
            }
            let finished_processing_queue_info_message = resource_service.get_resource_string_by_name("finished_processing_queue_info_message").unwrap();
            log::debug!("{}", finished_processing_queue_info_message);
            if let Ok(should_run_daemon_guard) = should_run_daemon_mutex.lock() {
                let waiting_message = resource_service.get_resource_string_by_name("waiting_for_should_run_daemon_guard_message").unwrap();
                log::debug!("{}", waiting_message);
                let temp_should_run_daemon_guard = cvar.wait(should_run_daemon_guard).unwrap();
                should_run_daemon = temp_should_run_daemon_guard.clone();
                let received_message = resource_service.get_resource_string_by_name("received_for_should_run_daemon_guard_message_template").unwrap();
                log::debug!("{}{}", received_message, should_run_daemon);
            }
        }
        let daemon_killed_message = resource_service.get_resource_string_by_name("daemon_killed_message").unwrap();
        log::debug!("{}", daemon_killed_message);
    }
}
