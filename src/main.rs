#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

// RawStr is a type provided by rocket to mark unvalidated strings
use rocket::http::RawStr;
use rocket::response::Redirect;
use rocket::Request;

use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use validator::validate_url;

// TODO: count and map should be stored in state struct
// lazy_static! {
//     static ref COUNT: Mutex<i32> = Mutex::new(0);
//     static ref MAP: Mutex<HashMap<i32, String>> = Mutex::new(HashMap::new());
// }

// fn get_count() -> i32 {
//     *COUNT.lock().unwrap()
// }

// fn increment_count() {
//     *COUNT.lock().unwrap() += 1;
// }

// fn put_mapping(url: &str) {
//     MAP.lock().unwrap().insert(get_count(), url.to_string());
// }

// fn get_mapping(key: i32) -> String {
//     match MAP.lock().unwrap().get(&key) {
//         Some(url) => format!["{}", url],
//         None => "/missing".to_string(),
//     }
// }

// new state struct
struct Mapping {
    count: i32,
    map: HashMap<i32, String>,
}

impl Mapping {
    fn new() -> Mapping {
        Mapping {
            count: 0,
            map: HashMap::new(),
        }
    }

    fn put(&mut self, url: &str) {
        self.count += 1;
        self.map.insert(self.count, url.to_string());
    }

    fn get(&self, key: i32) -> Option<&String> {
        self.map.get(&key)
    }

    fn count(&self) -> i32 {
        self.count
    }
}

lazy_static! {
    static ref MAPPING: Mutex<Mapping> = Mutex::new(Mapping::new());
}

#[get("/")]
fn index() -> &'static str {
    "welcome to my url shortener!"
}

#[get("/shorten?<url>")]
fn shortener(url: &RawStr) -> String {
    let url = url.to_string();

    if validate_url(&url) {
        MAPPING.lock().unwrap().put(&url.to_string());
        let count = MAPPING.lock().unwrap().count();
        format!["{} -- {}", count, url.to_string()]
    } else {
        "invalid url".to_string()
    }
}

#[get("/lookup?<key>")]
fn lookup(key: &RawStr) -> String {
    match key.to_string().parse::<i32>() {
        Ok(key) => match MAPPING.lock().unwrap().get(key) {
            Some(url) => url.to_string(),
            None => "we could not find that key 🤷".to_string(),
        },
        Err(error) => error.to_string(),
    }
}

#[get("/missing")]
fn missing() -> String {
    "this mapping doesn't exist 😲".to_string()
}

#[catch(404)]
fn not_found(req: &Request) -> Redirect {
    // get key from request withour leading "/"
    let key = &(req.uri().to_string())[1..];
    let key_int;

    match key.to_string().parse::<i32>() {
        Ok(k) => {
            key_int = k;
        }
        Err(_) => {
            return Redirect::to("/missing");
        }
    }

    match MAPPING.lock().unwrap().get(key_int) {
        Some(url) => Redirect::to(url.to_string()),
        None => Redirect::to("/missing"),
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, shortener, lookup, missing])
        .register(catchers![not_found])
        .launch();
}
