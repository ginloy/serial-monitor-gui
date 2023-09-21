use dioxus::{
    html::input_data::keyboard_types::{Key, Modifiers},
    prelude::*,
};
use log::*;

use crate::api::{self, Connection};

#[inline_props]
pub fn InputBox(
    cx: Scope,
    user_buffer: UseRef<String>,
    port_buffer: UseRef<String>,
    connection: UseRef<Connection>,
) -> Element {
    let inp = use_state(cx, || String::new());

    let write = |s: &str| {
        let s = format!("{s}\n");
        user_buffer.with_mut(|b| b.push_str(&s));
        cx.spawn({
            to_owned![connection];
            async move {
                if let Err(e) = connection.write().write(&s) {
                    error!("{:?}", e);
                }
            }
        })
    };

    render! {
        div {
            class: "d-flex gap-2",
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
                        if !inp.is_empty() && !event.modifiers().contains(Modifiers::SHIFT) && event.key() == Key::Enter {
                            write(&inp);
                            inp.set(String::new());
                        }
                    }
                }
                button {
                    class: "btn btn-primary bg-gradient",
                    onclick: move |_| {
                        if !inp.is_empty() {
                            write(&inp);
                            inp.set(String::new());
                        }
                    },
                    "Send"
                }
            },
            DownloadButton {
                user_buffer: user_buffer.clone(),
                port_buffer: port_buffer.clone(),
                titles: vec!["user".to_string(), connection.read().get_name().to_string()]
            }
        }
    }
}

#[inline_props]
fn DownloadButton(
    cx: Scope,
    user_buffer: UseRef<String>,
    port_buffer: UseRef<String>,
    titles: Vec<String>,
) -> Element {
    let is_downloading = use_state(cx, || false);
    let trigger_download = |_| {
        let content = vec![user_buffer.read().clone(), port_buffer.read().clone()];
        cx.spawn({
            to_owned![titles, is_downloading];
            async move {
                is_downloading.set(true);
                api::download(titles, content).await;
                is_downloading.set(false);
            }
        })
    };

    render! {
        if !*is_downloading.get() {
            rsx! {
                button {
                    class: "btn btn-primary",
                    onclick: trigger_download,
                    "Download"
                }
            }
        } else {
            rsx! {
                button {
                    class: "btn btn-primary d-flex align-items-center gap-1",
                    disabled: true,
                    div {
                        class: "spinner-border spinner-border-sm",
                    },
                    div { "Downloading..." },
                }
            }
        }
    }
}
