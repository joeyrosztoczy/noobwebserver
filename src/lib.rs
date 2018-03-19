use std::thread;
use std::net::TcpStream;

pub struct ThreadPool {
    threads: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    // Create a new Threadpool
    // 
    // Takes a # of threads for the pool, must be positive
    pub fn new(thread_limit: usize) -> ThreadPool {
        assert!(thread_limit > 0);

        let mut threads = Vec::with_capacity(thread_limit);
        ThreadPool { threads }
    }
    // Except we wanna take a function to execute
    // and execute it in a separate thread
    pub fn async<F>(&self, task: F) -> ()
        where F: FnOnce() + Send + 'static 
    {
        ()
    }
}
