use dioxus::{prelude::*, html::input_data::keyboard_types::{Modifiers, Key}};

use crate::api::Connection;

#[inline_props]
pub fn InputBox(cx: Scope, user_buffer: UseRef<String>, connection: UseRef<Connection>) -> Element {
    let inp = use_state(cx, || String::new());

    let write = |s: &str| {
        let s = format!("{s}\n");
        user_buffer.with_mut(|b| b.push_str(&s));
        connection.with_mut(|c| c.write(&s));
    };

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
        }
    }
}
