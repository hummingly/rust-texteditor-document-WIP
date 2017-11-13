#![windows_subsystem = "windows"]
#![allow(unknown_lints)]
extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate winapi;

mod utils;
mod document;

use document::{init_text, message_loop, TextDocument};

fn main() {
    let mut doc = TextDocument::initialized();
    init_text(&mut doc);

    message_loop();
}
