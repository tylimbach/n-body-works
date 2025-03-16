use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct TripleBuffer<T: Clone> {
    buffers: [Mutex<T>; 3],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
    swap_index: AtomicUsize,
}

#[allow(dead_code)]
impl<T: Clone> TripleBuffer<T> {
    pub fn new(initial_state: T) -> Self {
        Self {
            buffers: [
                Mutex::new(initial_state.clone()),
                Mutex::new(initial_state.clone()),
                Mutex::new(initial_state),
            ],
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(1),
            swap_index: AtomicUsize::new(2),
        }
    }

    pub fn get_write_buffer(&self) -> std::sync::MutexGuard<T> {
        let write_idx = self.write_index.load(Ordering::Acquire);
        self.buffers[write_idx].lock().unwrap()
    }

    pub fn get_read_buffer(&self) -> std::sync::MutexGuard<T> {
        let read_idx = self.read_index.load(Ordering::Acquire);
        self.buffers[read_idx].lock().unwrap()
    }

    pub fn commit(&self) {
        let write_idx = self.write_index.load(Ordering::Acquire);
        let swap_idx = self.swap_index.load(Ordering::Acquire);

        let write_buf = self.buffers[write_idx].lock().unwrap();
        let mut swap_buf = self.buffers[swap_idx].lock().unwrap();

        *swap_buf = write_buf.clone();
    }

    pub fn swap_read(&self) {
        let swap_idx = self.swap_index.load(Ordering::Acquire);
        self.read_index.store(swap_idx, Ordering::Release);
    }
}
