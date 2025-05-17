use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(left: i32, right: i32) -> i32 {
    console_log::init().expect("Could not init logger!!!");
    info!("Running wasm!!!");
    left + right
}
