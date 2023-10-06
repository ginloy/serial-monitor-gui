use dioxus::prelude::*;

use crate::{
    api::{self, Connection, DEFAULT_BR},
    components::{consoles::Consoles, input_box::InputBox, selector_row::SelectorRow},
    ports::PortInfo,
};

pub fn App(cx: Scope) -> Element {
    let available_ports = use_state(cx, || Vec::<PortInfo>::new());
    let connection = use_ref(cx, || Connection::new(DEFAULT_BR));
    let user_buffer = use_ref(cx, || Vec::<String>::new());
    let port_buffer = use_ref(cx, || Vec::<String>::new());

    let _ = use_coroutine(cx, |_: UnboundedReceiver<()>| {
        to_owned!(available_ports);
        async move {
            api::scan_ports(available_ports.clone()).await;
        }
    });
    render! {
        style { include_str!("../output.css") },
        body {
            "data-theme" : "synthwave",
            div {
                class: "h-screen flex flex-col gap-2 p-2",
                SelectorRow { available_ports: available_ports.clone(), connection: connection.clone(), buffer: port_buffer.clone() }
                Consoles { port_buffer: port_buffer.clone(), user_buffer: user_buffer.clone() }
                InputBox {
                    user_buffer: user_buffer.clone(),
                    connection: connection.clone(),
                    port_buffer: port_buffer.clone()
                }
            }
        }
    }
}
