use dioxus::{
    html::input_data::keyboard_types::{Key, Modifiers},
    prelude::*,
};
use log::*;

use crate::api::{self, Connection};

#[inline_props]
pub fn InputBox(
    cx: Scope,
    user_buffer: UseRef<Vec<String>>,
    port_buffer: UseRef<Vec<String>>,
    connection: UseRef<Connection>,
) -> Element {
    let inp = use_state(cx, || String::new());

    let write = |s: &str| {
        let s = format!("{s}\n");
        user_buffer.with_mut(|b| b.push(s.clone()));
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
            class: "flex flex-row gap-2",
            div {
                class: "join flex-1",
                input {
                    value: "{inp}",
                    class: "input input-bordered w-full join-item",
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
                    class: "btn btn-primary join-item",
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
    user_buffer: UseRef<Vec<String>>,
    port_buffer: UseRef<Vec<String>>,
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
                    class: "btn btn-primary btn-bordered",
                    disabled: true,
                    span {
                        class: "loading loading-spinner",
                    },
                    div { "Downloading..." },
                }
            }
        }
    }
}
