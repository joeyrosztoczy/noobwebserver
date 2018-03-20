use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;

// Workaround for taking ownership of the Box's to make the compiler happy
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new Threadpool
    ///
    /// Takes a # of threads for the pool, must be positive
    pub fn new(worker_limit: usize) -> ThreadPool {
        assert!(worker_limit > 0);
        // Open a multi-producer single consumer channel to pass messages to threads with
        let (sender, receiver) = mpsc::channel();
        // Shadow receiver into allowing multiple owners of the instance with an Atomic Ref
        // counting pointer aka Arc. Need to mutate the receiver, therefore create a Mutex
        // which will block threads trying to access the Job when its locked
        let receiver = Arc::new(Mutex::new(receiver));
        // Create a mutable vector to store workers in
        let mut workers = Vec::with_capacity(worker_limit);

        for worker_id in 0..worker_limit {
            // Can clone the receiver with Arc since its a threadsafe smart pointer
            workers.push(Worker::new(worker_id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }
    // Except we wanna take a function to execute
    // and execute it in a separate thread
    pub fn async<F>(&self, f: F) -> ()
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Try to acquire mutex with .lock(), panic if mutex is poisoned
                // recv() will receive a Job on the channel, can receive errors
                // if sender is down so a final unwrap is necessary
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a new job! Executing.", id);
                job.call_box();
            }
        });

        Worker { id, thread }
    }
}
