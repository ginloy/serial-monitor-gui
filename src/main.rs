#![allow(non_snake_case)]
mod api;
mod app;
mod handle;
mod ports;
mod components {
    pub mod consoles;
    pub mod input_box;
    pub mod selector_row;
}

use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    dioxus_desktop::launch(app::App);
}
