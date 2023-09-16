use crate::{ports, api};
use dioxus::{
    html::input_data::keyboard_types::{Key, Modifiers},
    prelude::*,
};
use fermi::*;

static APPSTATE: Atom<api::AppState> = Atom(|_| api::AppState::new());

pub fn App(cx: Scope) -> Element {
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
                class: "vh-100 container-fluid d-flex flex-column overflow-hidden",
                div {
                    class: "row mt-2 g-2",
                    div {
                        class: "col-12 col-sm-8",
                        menu_entry {}
                    }
                    div {
                        class: "col-10 col-sm-3",
                        baud_box {}
                    }
                    div {
                        class: "col-2 col-sm-1 d-flex justify-content-center align-items-center",
                        ConnectingSpinner {}
                    },
                }
                div {
                    class: "row mt-2 flex-grow-1 g-2",
                    id: "console-row",
                    min_height: "1rem",
                    div {
                        class: "col-12 col-md-6",
                        Console {}
                    }
                    div {
                        class: "col-12 col-md-6",
                        Console {}
                    }
                }
                div {
                    class: "row pb-2 mt-2",
                    div {
                        class: "col-md",
                        input_box {}
                    },
                }
            }
        }
    }
}

fn menu_entry(cx: Scope) -> Element {
    let available = use_state(cx, || Vec::<String>::new());
    let _ = use_coroutine(cx, |_: UnboundedReceiver<()>| {
        let available = available.to_owned();
        async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            loop {
                interval.tick().await;
                available.set(
                    ports::get_available_usb()
                        .into_iter()
                        .map(|(x, info)| {
                            format!(
                                "{}\t|\t{}\t|\t{}",
                                x,
                                info.manufacturer.unwrap_or(String::new()),
                                info.product.unwrap_or(String::new())
                            )
                        })
                        .collect(),
                );
            }
        }
    });
    render! {
        select {
            class: "form-select bg-secondary",
            onchange: |evt| println!("{:?}", evt),
            if available.is_empty() {
                rsx! { option { value: -1, "No ports detected" } }
            } else {
                rsx! { option { value: -1, "Select port" } }
            }
            available.iter().map(|x| rsx!{option {value: "{x}", "{x}"} })

        }
    }
}

fn Console(cx: Scope) -> Element {
    let create_eval = use_eval(cx);
    let eval = create_eval(
        r#"
        await dioxus.recv();
        var elements = document.querySelectorAll("[id='console']");
        for (var i = 0; i < elements.length; ++i) {
            elements[i].scrollTop = elements[i].scrollHeight;
        }
        // element.scrollTop = element.scrollHeight;
        "#,
    )
    .unwrap();

    eval.send("scroll".into()).unwrap();

    render! {
        div {
            class: "h-100 position-relative",
            textarea {
                id: "console",
                class: "form-control bg-secondary w-100 h-100",
                font_size: "0.875rem",
                readonly: true,
                resize: "none",
                (0..1000).map(|i| {
                    eval.send("test".into()).unwrap();
                    format!("The quick brown fox jumped over the lazy dog {i}\n")
                }).collect::<String>()
            }
            button {
                class: "btn btn-danger position-absolute bg-gradient",
                font_size: "0.9rem",
                top: "10px",
                right: "10px",
                "Clear"
            }
        }
    }
}

fn input_box(cx: Scope) -> Element {
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
                        println!("{}", inp);
                        inp.set(String::new());
                    } 
                }
            }
            button {
                class: "btn btn-primary bg-gradient",
                onclick: move |_| {
                    if !inp.is_empty() {
                        println!("{}", inp);
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
            "type": "number",
            min: "0",
            max: "200000",
            step: "100",
            placeholder: "baud rate",
            oninput: |event| inp.set(event.value.clone()),
        }
    }
}

fn green_circle(cx: Scope) -> Element {
    render! {
        div {
            class: "bg-gradient bg-primary h-75 rounded-circle",
            style: "aspect-ratio: 1 / 1;",
        }
    }
}

fn ConnectingSpinner(cx: Scope) -> Element {
    render! {
        div {
            class: "spinner-grow text-danger bg-gradient",
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
