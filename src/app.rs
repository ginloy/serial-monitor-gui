use dioxus::prelude::*;
use tokio_serial::UsbPortInfo;

use crate::{
    api::{self, Connection},
    components::{consoles::Consoles, input_box::InputBox, selector_row::SelectorRow},
};

pub fn App(cx: Scope) -> Element {
    let available_ports = use_state(cx, || Vec::<(String, UsbPortInfo)>::new());
    let connection = use_ref(cx, || Connection::new(9600));
    let user_buffer = use_ref(cx, || String::new());
    let port_buffer = use_ref(cx, || String::new());
    // to_owned![available_ports];
    let _ = use_coroutine(cx, |_: UnboundedReceiver<()>| {
        to_owned!(available_ports);
        async move {
            api::scan_ports(available_ports.clone()).await;
    }});
    render! {
        head {
            link {
                href: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.1/dist/css/bootstrap.min.css",
                rel: "stylesheet",
                integrity: "sha384-4bw+/aepP/YC94hEpVNVgiZdgIC5+VKNBQNGCHeKRQN+PtmoHDEXuppvnDJzQIu9",
                crossorigin:"anonymous"
            }
        }
        body {
            "data-bs-theme" : "dark",
            script {
                src: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.1/dist/js/bootstrap.bundle.min.js",
                integrity: "sha384-HwwvtgBNo3bZJJLYd8oVXjrBZt8cqVSpeBNS5n7C8IVInixGAoxmnlMuBnhbgrkm",
                crossorigin: "anonymous"
            }
            div {
                class: "vh-100 container-fluid d-flex flex-column",
                div {
                    class: "row pt-2",
                    div {
                        class: "col",
                        SelectorRow { available_ports: available_ports.clone(), connection: connection.clone(), buffer: port_buffer.clone() }
                    },
                }
                div {
                    class: "row flex-grow-1 pt-2",
                    min_height: "1rem",
                    div {
                        class: "col",
                        Consoles { port_buffer: port_buffer.clone(), user_buffer: user_buffer.clone() }
                    }
                }
                div {
                    class: "row pb-2",
                    div {
                        class: "col-md",
                        InputBox { user_buffer: user_buffer.clone(), connection: connection.clone() }
                    },
                }
            }
        }
    }
}
