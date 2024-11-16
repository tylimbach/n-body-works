use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct TripleBuffer<T: Clone> {
    buffers: [Mutex<T>; 3],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
    swap_index: AtomicUsize,
}

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

        let mut write_buf = self.buffers[write_idx].lock().unwrap();
        let mut swap_buf = self.buffers[swap_idx].lock().unwrap();

        *swap_buf = write_buf.clone();
    }

    pub fn swap_read(&self) {
        let swap_idx = self.swap_index.load(Ordering::Acquire);
        self.read_index.store(swap_idx, Ordering::Release);
    }
}

pub struct DoubleBuffer<T> {
    buffers: [T; 2],
    read_index: AtomicUsize,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn new(initial: T) -> Self {
        Self {
            buffers: [initial.clone(), initial],
            read_index: AtomicUsize::new(0),
        }
    }

    pub fn get_write_buffer(&mut self) -> &mut T {
        let write_index = 1 - self.read_index.load(Ordering::Acquire);
        &mut self.buffers[write_index]
    }

    pub fn commit(&mut self) {
        let write_index = 1 - self.read_index.load(Ordering::Acquire);
        let read_index = self.read_index.load(Ordering::Acquire);

        self.buffers[read_index] = self.buffers[write_index].clone();
    }

    pub fn get_read_buffer(&self) -> &T {
        &self.buffers[self.read_index.load(Ordering::Acquire)]
    }
}

pub struct DoubleBufferUnsafe<T: Clone> {
    buffers: [UnsafeCell<T>; 2],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

unsafe impl<T: Clone> Sync for DoubleBufferUnsafe<T> {}
