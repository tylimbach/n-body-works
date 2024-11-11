use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TripleBuffer<T> {
    buffers: [T; 3],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
    swap_index: AtomicUsize,
}

impl<T: Clone> TripleBuffer<T> {
    pub fn new(initial_state: T) -> Self {
        Self {
            buffers: [initial_state.clone(), initial_state.clone(), initial_state],
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(1),
            swap_index: AtomicUsize::new(2),
        }
    }

    pub fn get_write_buffer(&mut self) -> &mut T {
        &mut self.buffers[self.write_index.load(Ordering::Acquire)]
    }

    pub fn swap_write(&self) {
        let current_write = self.write_index.load(Ordering::Acquire);
        self.swap_index.swap(current_write, Ordering::Release);
    }

    pub fn swap_read(&self) {
        let current_read = self.read_index.load(Ordering::Acquire);
        self.swap_index.swap(current_read, Ordering::Release);
    }

    pub fn get_read_buffer(&self) -> &T {
        &self.buffers[self.read_index.load(Ordering::Acquire)]
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

impl<T: Clone> DoubleBufferUnsafe<T> {
    pub fn new(initial: T) -> Self
    where
        T: Clone,
    {
        Self {
            buffers: [
                UnsafeCell::new(initial.clone()),
                UnsafeCell::new(initial),
            ],
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(1),
        }
    }

    pub fn get_write_buffer(&self) -> &mut T {
        let write_idx = self.write_index.load(Ordering::Acquire);
        unsafe { &mut *self.buffers[write_idx].get() }
    }

    pub fn swap(&self) {
        let write_idx = self.write_index.load(Ordering::Acquire);
        let read_idx = self.read_index.swap(write_idx, Ordering::Release);
        self.write_index.store(1 - read_idx, Ordering::Release);
    }

    pub fn commit(&self) {
        let read_index = self.read_index.load(Ordering::Acquire);
        let write_index = 1 - read_index;

        unsafe {
            let write_buffer = &*self.buffers[write_index].get();
            let read_buffer = &mut *self.buffers[read_index].get();
            *read_buffer = (*write_buffer).clone();
        }
    }

    pub fn get_read_buffer(&self) -> &T {
        let read_idx = self.read_index.load(Ordering::Acquire);
        unsafe { &*self.buffers[read_idx].get() }
    }
}