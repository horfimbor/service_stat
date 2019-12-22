use std::io::Read;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use eventstore::{Connection, ReadStreamStatus, StreamSlice};
use eventstore::Slice;
use futures::Future;
use tiny_http::{Request, Response, Server};

fn main() {
    let connection = Connection::builder()
        .single_node_connection("172.28.1.1:1113".parse().unwrap());

    let nb_account = Arc::new(Mutex::new(0));
    let last_account = Arc::new(Mutex::new("".to_string()));

    let (nb, last) = (Arc::clone(&nb_account), Arc::clone(&last_account));
    thread::spawn(move || {
        loop {
            let result = read_stream(&connection, &nb);

            match result {
                eventstore::ReadStreamStatus::Success(slice) => match slice.events() {
                    eventstore::LocatedEvents::EndOfStream => {
                        break;
                    }

                    eventstore::LocatedEvents::Events { mut events, next } => {
                        let event = events.pop().unwrap();
                        let event = event.get_original_event().unwrap();
                        let obj: event_auth::AccountCreated = event.as_json().unwrap();


                        match next {
                            Some(n) => {
                                let mut nb = nb.lock().unwrap();
                                *nb = n;
                            }
                            None => {
                                let mut last = last.lock().unwrap();
                                *last = obj.name;
                                thread::sleep(Duration::from_secs(1))
                            }
                        }
                    }
                },

                eventstore::ReadStreamStatus::Error(error) => {
                    panic!("ReadStream error: {:?}", error);
                }
            }
        }
    });


    let server = Server::http("0.0.0.0:8001").unwrap();

    println!("listening on 8001");

    for request in server.incoming_requests() {
        println!("received request!\n, method: {:?}\n, url: {:?}\n, headers: {:?}\n",
                 request.method(),
                 request.url(),
                 request.headers(),
        );

        let url = request.url().to_string();
        let path = Path::new(&url);
        let file = fs::File::open(&path);

        if file.is_ok() {
            let mut response = tiny_http::Response::from_file(file.unwrap());
            let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/javascript"[..]).unwrap();

            response.add_header(header);
            add_cors(&request, &mut response);

            request.respond(response).unwrap();
        }else{
            let nb = nb_account.lock().unwrap();
            let last = last_account.lock().unwrap();
            let response_string = format!("{}:{}", *nb, *last);

            let mut response = Response::from_string(response_string);
            add_cors(&request, &mut response);
            request.respond(response).unwrap();
        }
    }
}

fn read_stream(connection: &Connection, nb: &Arc<Mutex<i64>>) -> ReadStreamStatus<StreamSlice> {
    let st = nb.lock().unwrap();
    let result =
        connection.read_stream("account")
            .start_from(*st)
            .max_count(1)
            .execute()
            .wait()
            .unwrap();
    result
}

//TODO move into a module
fn add_cors<T: Read>(request: &Request, response: &mut Response<T>) {

    //TODO check main domain

    for h in request.headers() {
        if h.field.equiv("Origin") {
            let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], h.value.as_bytes()).unwrap();
            response.add_header(header);
        }
    }

    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, GET"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Max-Age"[..], &b"86400"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Vary"[..], &b"Origin"[..]).unwrap();
    response.add_header(header);
    let header = tiny_http::Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"body, cache, Content-Type"[..]).unwrap();
    response.add_header(header);
}
