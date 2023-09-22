use dioxus::prelude::*;
use log::*;
use tokio_serial::UsbPortInfo;

use crate::api::{self, Connection};

#[inline_props]
pub fn SelectorRow(
    cx: Scope,
    available_ports: UseState<Vec<(String, UsbPortInfo)>>,
    connection: UseRef<Connection>,
    buffer: UseRef<Vec<String>>,
) -> Element {
    render! {
        div {
            class: "row g-2",
            div {
                class: "col-12 col-sm-8",
                Selector { available_ports: available_ports.clone(), connection: connection.clone(), buffer: buffer.clone() }
            },
            div {
                class: "col-10 col-sm-3",
                BaudSelector { connection: connection.clone() }
            }
            div {
                class: "col-2 col-sm-1 d-flex justify-content-center align-items-center",
                ConnectionIndicator { connection: connection.clone() }
            }
        }
    }
}

#[inline_props]
fn Selector(
    cx: Scope,
    available_ports: UseState<Vec<(String, UsbPortInfo)>>,
    connection: UseRef<Connection>,
    buffer: UseRef<Vec<String>>,
) -> Element {
    let prev_task: &UseState<Option<TaskId>> = use_state(cx, || None);
    let connect = move |e: Event<FormData>| {
        if let Some(id) = **prev_task {
            cx.remove_future(id);
        }
        connection.with_mut(|c| c.close());
        if e.value != "none" {
            let id = cx.push_future({
                to_owned![connection, buffer, prev_task];
                async move {
                    api::connect(connection.clone(), &e.value).await;
                    info!("Connected to {}", &e.value);
                    api::read(connection.clone(), buffer.clone()).await;
                    prev_task.set(None);
                }
            });
            prev_task.set(Some(id));
        }
    };

    render! {
        div {
            class: "form-floating",
            select {
                class: "form-select",
                onchange: connect,
                if available_ports.is_empty() {
                    rsx! { option { value: "none", "No ports detected" } }
                } else {
                    rsx! { option { value: "none" ,"Select port" } }
                }
                available_ports.iter().map(|(x, inf)| rsx!{ option {
                    value: "{x}",
                    format!("{}\t|\t{}\t|\t{}", x, inf.manufacturer.clone().unwrap_or(String::new()), inf.product.clone().unwrap_or(String::new()))
                }})
            },
            label {
                "Port"
            },
        }
    }
}

#[inline_props]
fn BaudSelector(cx: Scope, connection: UseRef<Connection>) -> Element {
    let inp = use_state(cx, || format!("{}", api::DEFAULT_BR));
    let set_br = |s: &str| {
        inp.set(s.to_string());
        match str::parse::<u32>(s) {
            Ok(x) => match connection.with_mut(|c| c.set_baud_rate(x)) {
                Ok(_) => {
                    info!("Baud rate set to {x}");
                }
                Err(e) => {
                    error!("Failed to set baud_rate to {x} due to {e}");
                }
            },
            Err(_) => {
                warn!("Not a valid number");
            }
        };
    };
    render! {
        div {
            class: "form-floating",
            input {
                value: "{inp}",
                class: "form-control",
                r#type: "number",
                min: "0",
                max: "200000",
                step: "100",
                placeholder: "baud rate",
                oninput: move |event| {
                    set_br(&event.value);
                }
            },
            label {
                "Baud Rate"
            },
        }
    }
}

#[inline_props]
fn ConnectionIndicator(cx: Scope, connection: UseRef<Connection>) -> Element {
    render! {
        if connection.with(|c| c.is_connected()) {
            rsx! {
                ConnectedSpinner {}
            }
        } else {
            rsx! {
                ConnectingSpinner {}
            }
        }
    }
}

fn ConnectingSpinner(cx: Scope) -> Element {
    render! {
        div {
            class: "spinner-border text-primary",
            role: "status",
        }
    }
}

fn ConnectedSpinner(cx: Scope) -> Element {
    render! {
        div {
            class: "spinner-grow text-success bg-gradient",
            role: "status",
            style: "animation-duration: 2s;"
        }
    }
}
