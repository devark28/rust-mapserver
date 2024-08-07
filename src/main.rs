// cSpell:disable
use std::io::{prelude::*, BufReader, Write}; //, stdout
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;
use std::{fmt, thread};
use std::sync::{Arc, Mutex};

use rand::Rng;
use serde::Serialize;
use threadpool::ThreadPool;

const MAP_WIDTH: i32 = 20;
const MAP_HEIGHT: i32 = 10;
const MAX_NUM_AIRCRAFTS: i32 = 10;
const MIN_NUM_AIRCRAFTS: i32 = 10;

#[derive(Clone, Debug, Serialize)]
enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::N => write!(f, "↑ "),
            Direction::NE => write!(f, "↗ "),
            Direction::E => write!(f, "→ "),
            Direction::SE => write!(f, "↘︎ "),
            Direction::S => write!(f, "↓ "),
            Direction::SW => write!(f, "↙ "),
            Direction::W => write!(f, "← "),
            Direction::NW => write!(f, "↖︎ "),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct Flight {
    id: String,
    x: i32,
    y: i32,
    direction: Direction,
}

fn main() {
    let mut traffic_data: Vec<Flight> = Vec::new();

    generate_map(&mut traffic_data);
    // dbg!(&traffic_data);
    // draw_char_map(&traffic_data);

    let (req_sender, req_receiver) = mpsc::channel::<()>();
    let (data_sender, data_receiver) = mpsc::channel::<Vec<Flight>>();

    // periodically move the aircrafts (animation thread)
    let handle = thread::spawn(move || {
        loop {
            // check if data has been requested
            if let Ok(_) = req_receiver.try_recv() {
                data_sender.send(traffic_data.clone()).unwrap();
            }

            move_aircrafts(&mut traffic_data);
            // draw_char_map(&traffic_data);
            sleep(Duration::from_millis(900));
        }
    });

    // running the REST API server
    let listner = TcpListener::bind("localhost:3000").expect("Unable to bind to port 3000");
    println!("Now listening to port 3000...");

    let thread_pool = ThreadPool::new(10);
    let data_mutex = Arc::new(Mutex::new(data_receiver));

    for stream_result in listner.incoming() {
        let req_sender_clone = req_sender.clone();
        let data_mutex_clone = data_mutex.clone();

        if let Ok(stream) = stream_result {
            thread_pool.execute(move || {
                process_stream(stream, &req_sender_clone, data_mutex_clone);
            })
        }
    }
    handle.join().unwrap();
}

fn add_new_flight(data_set: &mut Vec<Flight>) {
    let mut rng = rand::thread_rng();
    let letter1: char = rng.gen_range(b'A'..b'Z') as char;
    let letter2: char = rng.gen_range(b'A'..b'Z') as char;
    let number: u32 = rng.gen_range(10..9999);
    let new_id = format!("{}{}{:02}", letter1, letter2, number);

    // generate random x, y coordinates
    let new_x = rand::thread_rng().gen_range(0..MAP_WIDTH);
    let new_y = rand::thread_rng().gen_range(0..MAP_HEIGHT);

    // generate a random direction
    let dir = rand::thread_rng().gen_range(0..8);
    let new_dir = match dir {
        0 => Direction::N,
        1 => Direction::NE,
        2 => Direction::E,
        3 => Direction::SE,
        4 => Direction::S,
        5 => Direction::SW,
        6 => Direction::W,
        7 => Direction::NW,
        _ => Direction::N,
    };

    data_set.push(Flight {
        id: new_id,
        x: new_x,
        y: new_y,
        direction: new_dir,
    });
}

// fn draw_char_map(data_set: &[Flight]) {
//     let mut lock = stdout().lock();
//     for y in 0..(MAP_HEIGHT) {
//         write!(lock, " ").unwrap();
//         for _ in 0..(MAP_WIDTH) {
//             write!(lock, "-- ").unwrap();
//         }
//         write!(lock, "\r\n").unwrap();
//         for x in 0..(MAP_WIDTH) {
//             write!(lock, "|").unwrap();
//             // is there an aircraft in this box's coordinates?
//             let ufo = data_set
//                 .iter()
//                 .find(|flight| flight.x == x && flight.y == y);
//             match ufo {
//                 None => write!(lock, "  ").unwrap(),
//                 Some(f) => write!(lock, "{}", f.direction.to_string()).unwrap(),
//             }
//         }
//         write!(lock, "|\r\n").unwrap();
//     }
//     // print the bottom line
//     for _ in 0..(MAP_WIDTH) {
//         write!(lock, " --").unwrap();
//     }
//     write!(lock, "\r\n").unwrap();
// }

fn generate_map(data_set: &mut Vec<Flight>) {
    let num_aircrafts = rand::thread_rng().gen_range(MIN_NUM_AIRCRAFTS..(MAX_NUM_AIRCRAFTS + 1));
    for _ in 0..num_aircrafts {
        add_new_flight(data_set);
    }
}

fn move_aircrafts(data_set: &mut [Flight]) {
    for i in 0..data_set.iter().count() {
        match &data_set[i].direction {
            Direction::N => {
                data_set[i].y = data_set[i].y - 1;
                if data_set[i].y < 0 {
                    data_set[i].y = MAP_HEIGHT - 1;
                }
            }

            Direction::NE => {
                data_set[i].y = data_set[i].y - 1;
                if data_set[i].y < 0 {
                    data_set[i].y = MAP_HEIGHT - 1;
                }
                data_set[i].x = data_set[i].x + 1;
                if data_set[i].x >= MAP_WIDTH {
                    data_set[i].x = 0;
                }
            }

            Direction::E => {
                data_set[i].x = data_set[i].x + 1;
                if data_set[i].x >= MAP_WIDTH {
                    data_set[i].x = 0;
                }
            }

            Direction::SE => {
                data_set[i].x = data_set[i].x + 1;
                if data_set[i].x >= MAP_WIDTH {
                    data_set[i].x = 0;
                }
                data_set[i].y = data_set[i].y + 1;
                if data_set[i].y >= MAP_HEIGHT {
                    data_set[i].y = 0;
                }
            }

            Direction::S => {
                data_set[i].y = data_set[i].y + 1;
                if data_set[i].y >= MAP_HEIGHT {
                    data_set[i].y = 0;
                }
            }

            Direction::SW => {
                data_set[i].y = data_set[i].y + 1;
                if data_set[i].y >= MAP_HEIGHT {
                    data_set[i].y = 0;
                }
                data_set[i].x = data_set[i].x - 1;
                if data_set[i].x < 0 {
                    data_set[i].x = MAP_WIDTH - 1;
                }
            }

            Direction::W => {
                data_set[i].x = data_set[i].x - 1;
                if data_set[i].x < 0 {
                    data_set[i].x = MAP_WIDTH - 1;
                }
            }

            Direction::NW => {
                data_set[i].x = data_set[i].x - 1;
                if data_set[i].x < 0 {
                    data_set[i].x = MAP_WIDTH - 1;
                }
                data_set[i].y = data_set[i].y - 1;
                if data_set[i].y < 0 {
                    data_set[i].y = MAP_HEIGHT - 1;
                }
            }
        }
    }
}

fn process_stream(
    mut stream: TcpStream,
    data_requester: &Sender<()>,
    data_receiver: Arc<Mutex<Receiver<Vec<Flight>>>>,
) {
    let http_request = read_http_request(&mut stream);
    if http_request.iter().count() <= 0 || http_request[0].len() < 6 || &http_request[0][..6] != "GET / " {
        return;
    }
    let latest_traffic_data = get_latest_traffic_data(data_requester, data_receiver);
    send_http_response(&mut stream, &latest_traffic_data);
}

fn read_http_request(stream: &mut TcpStream) -> Vec<String> {
    let buffer_reader = BufReader::new(stream);
    let http_request = buffer_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    // println!("Received an http request!");
    return http_request;
}

fn send_http_response(stream: &mut TcpStream, data: &Option<Vec<Flight>>) {
    let respond_line = "HTTP/1.1 200 OK";

    let unwrapped_data: &Vec<Flight> = match data {
        Some(data) => &data,
        None => &vec![],
    };
    let serialization_result = serde_json::to_string(unwrapped_data);
    let payload = &*match serialization_result {
        Ok(data) => data,
        _ => String::from("[]"),
    };

    let content_length = payload.len();
    let content_type = "application/json";

    let headers = &*vec![
        format!("Content-Length: {content_length}"),
        "Access-Control-Allow-Origin: *".into(),
        format!("Content-Type: {content_type}"),
        "".into(),
    ]
    .join("\r\n");

    let http_response = vec![respond_line, headers, payload].join("\r\n");
    stream.write_all(http_response.as_bytes()).unwrap();
}

fn get_latest_traffic_data(
    data_requester: &Sender<()>,
    data_receiver: Arc<Mutex<Receiver<Vec<Flight>>>>,
) -> Option<Vec<Flight>> {
    data_requester.send(()).unwrap();
    match data_receiver.lock().unwrap().recv_timeout(Duration::from_millis(500)) {
        Ok(data) => Some(data),
        _ => None,
    }
}
