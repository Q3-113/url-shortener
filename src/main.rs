#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

// RawStr is a type provided by rocket to mark unvalidated strings
use rocket::http::RawStr;
use rocket::response::Redirect;
use rocket::Request;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use validator::validate_url;

struct Mapping {
    count: u32,
    map: HashMap<String, String>,
}

impl Mapping {
    fn new() -> Mapping {
        Mapping {
            count: 0,
            map: HashMap::new(),
        }
    }

    fn put(&mut self, url: &str) -> String {
        // TODO: deal with hash collisions
        self.count += 1;
        let hash = calculate_hash(&format!["{}{}", url, self.count]);
        let id = base_62::encode(&hash.to_le_bytes());
        self.map.insert(id.clone(), url.to_string());
        
        id
    }  

    fn get(&self, key: String) -> Option<&String> {
        self.map.get(&key)
    }
}

// TODO: eventually use a persistant key, value store
lazy_static! {
    static ref MAPPING: Arc<Mutex<Mapping>> = Arc::new(Mutex::new(Mapping::new()));
}

#[get("/")]
fn index() -> &'static str {
    "welcome to my url shortener 👋"
}

#[get("/shorten?<url>")]
fn shortener(url: &RawStr) -> String {
    let url = urlencoding::decode(&url.to_string()).unwrap();

    if validate_url(&url) {
        // it seems to be fine to panick on a poisened lock
        let key = MAPPING.lock().unwrap().put(&url.to_string());

        format!["{} => {}", url, key]
    } else {
        "invalid url 😕".to_string()
    }
}

#[get("/lookup?<key>")]
fn lookup(key: &RawStr) -> String {
    match MAPPING.lock().unwrap().get(key.to_string()) {
        Some(url) => url.to_string(),
        None => "we could not find that key 🤷".to_string(),
    }
}

#[get("/missing")]
fn missing() -> String {
    "this mapping doesn't exist 😲".to_string()
}

#[catch(404)]
fn not_found(req: &Request) -> Redirect {
    // get key from request withour leading "/"
    let key = (req.uri().to_string())[1..].to_string();

    match MAPPING.lock().unwrap().get(key) {
        Some(url) => Redirect::to(url.to_string()),
        None => Redirect::to("/missing"),
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, shortener, lookup, missing])
        .register(catchers![not_found])
        .launch();
}
