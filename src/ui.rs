use crate::{api, ports};
use api::AppState;
use dioxus::{
    html::input_data::keyboard_types::{Key, Modifiers},
    prelude::*,
};
use log::{info,error};
use tokio::time::{self, timeout, Duration};

pub fn App(cx: Scope) -> Element {
    let app_state = use_ref(cx, || AppState::new());
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
                    class: "row mt-2 g-2",
                    div {
                        class: "col-12 col-sm-8",
                        menu_entry { app_state: app_state }
                    }
                    div {
                        class: "col-10 col-sm-3",
                        baud_box {}
                    }
                    div {
                        class: "col-2 col-sm-1 d-flex justify-content-center align-items-center",
                        ConnectedIndicator {app_state: app_state}
                    },
                }
                div {
                    class: "row mt-2 flex-grow-1 g-2",
                    id: "console-row",
                    min_height: "1rem",
                    div {
                        class: "col-12 col-md-6",
                        Console { id: 0, app_state: app_state }
                    }
                    div {
                        class: "col-12 col-md-6",
                        Console { id: 2,  app_state: app_state}
                    }
                }
                div {
                    class: "row pb-2 mt-2",
                    div {
                        class: "col-md",
                        input_box { app_state: app_state }
                    },
                }
            }
        }
    }
}

#[inline_props]
fn menu_entry<'a>(cx: Scope, app_state: &'a UseRef<AppState>) -> Element {
    // let app_state_read = app_state.write();
    let available = app_state.with(|x| x.get_available_ports().clone());
    let refresh_handle = use_ref(cx, || 0);
    let _ = use_future(cx, (), |_| {
        let refresh_handle = refresh_handle.to_owned();
        async move {
            let mut interval = time::interval(api::SCAN_FREQ);
            loop {
                interval.tick().await;
                refresh_handle.needs_update();
            }
        }
    });

    let connect_handle: &UseState<Option<TaskId>> = use_state(cx, || None);
    let connect = {
        let app_state = app_state.to_owned();
        move |e: Event<FormData>| {
            if e.value == "none" {
                app_state.with_mut(|x| x.disconnect());
                return;
            }
            cx.spawn({
                let app_state = app_state.to_owned();
                let mut interval = time::interval(Duration::from_millis(5000));
                let evt = e.clone();
                async move {
                    let future = || async move {
                        loop {
                        interval.tick().await;
                       match app_state.write().connect(&evt.value, 9600) {
                            Ok(_) => break,
                            Err(_) => {
                            }
                        }
                    }};
                    if let Err(_) = timeout(Duration::from_millis(10000), future()).await {
                        error!("Attempt to connect to {} time out", e.value);
                    } else {
                    info!("Connected to {}", e.value);
                    }
                }
            });
        }
    };
    render! {
        select {
            class: "form-select bg-secondary",
            onchange: connect, //|evt| println!("{:?}", evt),
            if available.is_empty() {
                rsx! { option { value: "none", "No ports detected" } }
            } else {
                rsx! { option { value: "none" ,"Select port" } }
            }
            available.iter().map(|(x, inf)| rsx!{ option {
                value: "{x}",
                format!("{}\t|\t{}\t|\t{}", x, inf.manufacturer.clone().unwrap_or(String::new()), inf.product.clone().unwrap_or(String::new()))
            }})
        }
    }
}

#[inline_props]
fn Console<'a>(cx: Scope<'a>, id: usize, app_state: &'a UseRef<AppState>) -> Element {
    let element_id = format!("console_{id}");
    let eval = use_eval(cx).clone();
    let script = format!(
        r#"
        var elements = document.querySelectorAll("[id='{element_id}']");
        for (var i = 0; i < elements.length; ++i) {{
            elements[i].scrollTop = elements[i].scrollHeight;
        }}
        // element.scrollTop = element.scrollHeight;
        "#
    );
    cx.push_future(async move {
        eval(script.as_ref()).unwrap();
    });
    let second_handle = app_state.to_owned();
    let app_state = app_state.read();
    let content = match id {
        0 => app_state.get_input_text(),
        _ => app_state.get_output_text(),
    };
    let content = &*content;

    render! {
        div {
            class: "h-100 position-relative",
            textarea {
                id: "{element_id}",
                class: "form-control bg-secondary w-100 h-100",
                font_size: "0.875rem",
                readonly: true,
                resize: "none",
                "{content}"
            }
            button {
                class: "btn btn-danger position-absolute bg-gradient",
                font_size: "0.9rem",
                top: "10px",
                right: "10px",
                onclick: move |_| {
                    match id {
                        0 => second_handle.with_mut(|x| x.clear_input()),
                        _ => second_handle.with_mut(|x| x.clear_output())
                    }
                },
                "Clear"
            }
        }
    }
}

#[inline_props]
fn input_box<'a>(cx: Scope, app_state: &'a UseRef<AppState>) -> Element {
    let inp = use_state(cx, || String::new());
    render! {
        div {
            class: "input-group",
            input {
                value: "{inp}",
                class: "form-control bg-gradient",
                spellcheck: "false",
                oninput: move |event| {
                   inp.set(event.value.clone());
                },
                onkeypress: move |event| {
                    // println!("{:?}", event);
                    if !inp.is_empty() && !event.modifiers().contains(Modifiers::SHIFT) && event.key() == Key::Enter {
                        app_state.with_mut(|x| x.append_input(format!("{inp}\n").as_str()));
                        inp.set(String::new());
                    }
                }
            }
            button {
                class: "btn btn-primary bg-gradient",
                onclick: move |_| {
                    if !inp.is_empty() {
                        app_state.with_mut(|x| x.append_input(format!("{inp}\n").as_str()));
                        inp.set(String::new());
                    }
                },
                "Send"
            }
        }
    }
}

fn baud_box(cx: Scope) -> Element {
    let inp = use_state(cx, || "9600".to_string());
    render! {
        input {
            value: "{inp}",
            class: "form-control bg-secondary",
            r#type: "number",
            min: "0",
            max: "200000",
            step: "100",
            placeholder: "baud rate",
            oninput: |event| inp.set(event.value.clone()),
        }
    }
}

#[inline_props]
fn ConnectedIndicator<'a>(cx: Scope, app_state: &'a UseRef<AppState>) -> Element {
    let is_connected = app_state.with(|x| x.is_connected());
    let refresh_handle = use_ref(cx, || 0);
    let _ = use_future(cx, (), |_| {
        let refresh_handle = refresh_handle.to_owned();
        async move {
            let mut interval = time::interval(api::SCAN_FREQ);
            loop {
                interval.tick().await;
                refresh_handle.needs_update();
            }
        }
    });
    render! {
        if is_connected {
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
