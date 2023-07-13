use project::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();//this binds the TCP listener to the local host address port and since we are unwrapping it means there is a result returned. If it fails it will thus panic and if it succeeds it will return the value
    let pool = ThreadPool::new(4); //we create a threadpool of 4 threads available for handling tasks. We dont want to spawn unlimited for safety (or "leak" too much power to the requester). These execute concurrently.
  
//this for loop processes each connection and produces the multiple streams to handle
    for stream in listener.incoming().take(2) {//returns an iterator with the connections received on this listener
        let stream = stream.unwrap();//result of TCP stream

      //closure takes ownership of environment variables which here for the threadpool we want to execute 1 thread at a time per request. Then feed the requests through our connection in the buffer. Here whatever is available in pool will be able to execute requests
        pool.execute(|| {
            handle_connection(stream);
        });
      //ultimately it takes a closure and gives it to a thread in the pool to run 
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {//mutable because its internal state might change
    let mut buffer = [0; 1024];//buffer to hold data read in
    stream.read(&mut buffer).unwrap();//create the lines of request that the browser will send to the server. Unwrap since we want a result and panics if what is expected is not returned

    let get = b"GET / HTTP/1.1\r\n"; // get request of the home url
    let sleep = b"GET /sleep HTTP/1.1\r\n"; // get request of the url/sleep 

  //creating a touple with the status and filename
  //in the same line we are checking an expression denoted by the "if" to see if the buffer has a status code we intend then we pull the hello.html contents we read from the buffer otherwise we would just be handling 1 request at a time making it single threaded and sequential (not the purpose in this multithreaded asynchronous execution we desire)
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) { //we check for the request to sleep (url/sleep) so if it is received then the server will sleep for 5 seconds before rendering the home page (hello.html)
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else { //if the conditional is not the 200 status then we return the 404 touple
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();//assign html file to content
  
//this is where you define the response. The http request needs this format which can also be written as \r\n to seperate and collect the request data where \r is a carraige return and \n is the line feed (CRLF)
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();//takes our reference to the vector and sends the bytes to the connection
    stream.flush().unwrap(); //you don't want to use println! to report errors. Either you should return the error from your function and let the caller deal with it, or you should use panic! to abort that thread and potentially the process. flush().unwrap() combo addresses these concerns of formatting and error handling
}
