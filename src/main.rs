#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

// RawStr is a type provided by rocket to mark unvalidated strings
use rocket::http::RawStr;

// TODO: global hashmap to store mappings -> we need singleton + mutex
// https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton

// TODO: validate the urls passed to the adder endpoint

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/add?<a>&<b>")]
fn adder(a: &RawStr, b: &RawStr) -> String {
    let a: i32 = a.parse::<i32>().unwrap();
    let b: i32 = b.parse::<i32>().unwrap();
    let sum: i32 = a + b;
    format!["{}", sum]
}

#[get("/shorten?<url>")]
fn shortener(url: &RawStr) -> String {
    format!["{}", url.to_string()]
}

// TODO: lookup function: hash -> url

fn main() {
    rocket::ignite().mount("/", routes![index, adder, shortener])
    .launch();
}