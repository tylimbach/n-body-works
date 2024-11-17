pub fn start_simulation_thread(
    state_buffer: Arc<TripleBuffer<SimulationState>>,
    simulation_frames: Arc<Mutex<FrameQueue>>,
) {
    std::thread::spawn(move || {
        let mut last_time = std::time::Instant::now();

        loop {
            let now = std::time::Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            log::info!("Simulation update dt: {:.6} seconds", dt);

            {
                let mut simulation_state = state_buffer.get_write_buffer();
                simulation_state.update(0.1);
            }

            // commit after we drop the lock
            state_buffer.commit();

            {
                simulation_frames.lock().unwrap().record_frame();
            }
        }
    });
}
