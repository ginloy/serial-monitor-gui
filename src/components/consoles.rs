use dioxus::prelude::*;

#[inline_props]
pub fn Consoles(cx: Scope, port_buffer: UseRef<String>, user_buffer: UseRef<String>) -> Element {
    render! {
        div {
            class: "row g-2 h-100",
            div {
                class: "col-12 col-md",
                Console { id: 0, buffer: user_buffer.clone() }
            },
            div {
                class: "col-12 col-md",
                Console { id: 1, buffer: port_buffer.clone() }
            },
        }
    }
}

#[inline_props]
fn Console(cx: Scope, id: usize, buffer: UseRef<String>) -> Element {
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

    let content = buffer.read();

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
                        0 => buffer.with_mut(|x| x.clear()),
                        _ => buffer.with_mut(|x| x.clear())
                    }
                },
                "Clear"
            }
        }
    }
}
