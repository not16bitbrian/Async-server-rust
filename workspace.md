First version before checking status codes when sending requests (Single Threaded w/ comments)
--------------------------------------------------------------------------------------------

fn handle_connection(mut stream: TcpStream) { //mutable because its internal state might change
    let buf_reader = BufReader::new(&mut stream); //buffer to hold data read in

  //create the lines of request that the browser will send to the server. Evidently from below it will be stored in a vector line by line where since we want a result it will need to be unwrapped
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
  


  let status_line = "HTTP/1.1 200 OK"; //status code for success otherwise the error handling is done from unwrap's panic funciton
  let contents = fs::read_to_string("hello.html").unwrap(); //assign html file to content
  let length = contents.len(); //content length of html file needed to allocate the space to write hello.html 
  
  let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"); //defining our response

  //takes our reference to the vector and sends the bytes to the connection
  stream.write_all(response.as_bytes()).unwrap();
}


Final version w refactoring of Single threaded with comments
-------------------------------------------------------------------------------------------

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap(); 
//this binds the TCP listener to the local host address port and since we are unwrapping it means there is a result returned. If it fails it will thus panic and if it succeeds it will return the value

  //this for loop processes each connection and produces the multiple streams to handle
    for stream in listener.incoming() {//returns an iterator with the connections received on this listener
        let stream = stream.unwrap();
  //result of TCP stream
        //println!("Connection established!");
      handle_connection(stream); //passing our stream to it
    }
}

fn handle_connection(mut stream: TcpStream) { //mutable because its internal state might change
    let buf_reader = BufReader::new(&mut stream); //buffer to hold data read in
    let request_line = buf_reader.lines().next().unwrap().unwrap();
  //create the lines of request that the browser will send to the server. Evidently from below it will be stored in a vector line by line where since we want a result it will need to be unwrapped
  
  
//status code for success otherwise the error handling is done from unwrap's panic funciton in this updated version we can send different requests based on different status codes. the previous version can be found in the worspace.md file
    if request_line == "GET / HTTP/1.1" {
        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("hello.html").unwrap(); //assign html file to content
        let length = contents.len(); //content length of html file needed to allocate the space to write hello.html 

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"//this is where you define the response. The http request needs this format which can also be written as \r\n to seperate and collect the request data where \r is a carraige return and \n is the line feed (CRLF)
        );
//takes our reference to the vector and sends the bytes to the connection
        stream.write_all(response.as_bytes()).unwrap();
    } 

      //format is the same but is for when when the status code is not the normal for our intended url
    else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("404.html").unwrap(); //reading new html file we created when the status code returned is for a page resource we did not establish
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        stream.write_all(response.as_bytes()).unwrap();
    }
}