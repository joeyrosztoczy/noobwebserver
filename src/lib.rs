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

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
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

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Shut down all workers!");
        // Send a terminate message down channel for each worker
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);
            // move the Some() value of the thread out and replace with None
            // so that we safely have ownership to join the thread, if let
            // is a shorter destructure syntax than a match
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>, // Wrap in an option for graceful shutdown
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Try to acquire mutex with .lock(), panic if mutex is poisoned
                // recv() will receive a Job on the channel, can receive errors
                // if sender is down so a final unwrap is necessary
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);

                        job.call_box();
                    }
                    Message::Terminate => {
                        println!("Worker {} instructed to shut down", id);

                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
