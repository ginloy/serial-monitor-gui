use dioxus::prelude::*;
use log::*;

use crate::{
    api::{self, Connection},
    ports::PortInfo,
};

#[inline_props]
pub fn SelectorRow(
    cx: Scope,
    available_ports: UseState<Vec<PortInfo>>,
    connection: UseRef<Connection>,
    buffer: UseRef<Vec<String>>,
) -> Element {
    render! {
        div {
            class: "flex flex-row gap-2 items-center",
                Selector { available_ports: available_ports.clone(), connection: connection.clone(), buffer: buffer.clone() }
                BaudSelector { connection: connection.clone() }
        }
    }
}

#[inline_props]
fn Selector(
    cx: Scope,
    available_ports: UseState<Vec<PortInfo>>,
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
            class: "indicator flex-1",
            span { class:"indicator-item indicator-start", ConnectionIndicator{ connection: connection.clone() } },
            select {
                class: "select select-bordered w-full",
                onchange: connect,
                if available_ports.is_empty() {
                    rsx! { option { value: "none", "No ports detected" } }
                } else {
                    rsx! { option { value: "none" ,"Select port" } }
                }
                available_ports.iter().map(|inf| rsx!{ option {
                    value: inf.name(),
                    format!("{}\t|\t{}\t|\t{}", inf.name(), inf.manufacturer(), inf.product())
                }})
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
            input {
                value: "{inp}",
                class: "input input-bordered",
                r#type: "number",
                min: "0",
                max: "200000",
                step: "100",
                placeholder: "baud rate",
                oninput: move |event| {
                    set_br(&event.value);
                }
            },
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
            class: "loading loading-spinner loading-xs",
            role: "status",
        }
    }
}

fn ConnectedSpinner(cx: Scope) -> Element {
    render! {
        div {
            class: "badge badge-success badge-xs",
            role: "status",
        }
    }
}
