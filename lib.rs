use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool { //create a channel 
    workers: Vec<Worker>, //holds receiving side
    sender: Option<mpsc::Sender<Job>>, //sending part of asynchronous channel
}
//Job will hold the closures we want to send down the channel
type Job = Box<dyn FnOnce() + Send + 'static>;
//box is a pointer type for heap allocation that captures the environment denoted by the closure of FnOnce Send and static

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {//Set size of a ThreadPool. The size of a thread pool is the number of worker threads spawned.
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
//here channel sends a touple that will be asynchronous NONBLOCKING 
//The multi producer single consumer module allows communication between channels in a First In First Out procedure
        let receiver = Arc::new(Mutex::new(receiver));
//mutex allows only one thread to access data. It must signal then lock so there is not multiple writers. It then unlocks so other threads can acquire it.
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }//atomic reference counter is used to share ownership between threads. By cloning the receiver we create a reference pointer for the location  of the value in the heap and also incrementing the RC.
      //It is dropped when the last reference pointer is out of scope. ALl the arc instances are dealt with in the spawned threads

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    { //takes the environment
        let job = Box::new(f);
      //creates the job to send down the channel denoted in its own environment from above
        self.sender.as_ref().unwrap().send(job).unwrap();
      //attempts to send value as reference to asynchronous channel returning it if it cannot be sent
    }
}

impl Drop for ThreadPool { //must manually call drop (the deconstructer) to free up memory when resources are no longer being used. Rust handles this most of the time automatically but is necessary here since Rust technically does not have garbage collection.
    fn drop(&mut self) {
        drop(self.sender.take());
//dropping sender closes the channel so no more messages will be sent
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
//workers no longer receive requests as theyre dropped
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }//all the calls left in the infinite loop will return err
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
  //joinhandle detaches the thread when it is dropped. This also makes it so the ability to join a thread is a uniquely owned permission
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
      //pass the receiving end of the channel into a new worker and use it inside a closure. Cannot clone but we do need to distribute therefore we use our automic reference counter to send our message of execution when a thread has the lock. "move" converts the variables captured by reference to variables captured by value to give ownership of useful data in the closure.
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message { //match checks which message to print based on the workers status
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
//Option replaces the Some variant with None. We’re using it to destructure the Some and get the thread; then we call join on the thread. If a worker’s thread is already None, we know that worker thread is already cleaned therefore nothing happens 
        }
    }
}
