#![no_std]

extern crate alloc;

use alloc::{format, string::String};
use wasm_bindgen::prelude::*;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let p1 = document.create_element("p")?;
    p1.set_inner_html("Hello from Rust!");

    let p2 = document.create_element("p")?;
    p2.set_inner_html("Hello from Rust again!");

    body.append_child(&p1)?;
    body.append_child(&p2)?;

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[wasm_bindgen]
pub fn say_hi(user: String) -> String {
    format!("Hello, {}", user)
}
