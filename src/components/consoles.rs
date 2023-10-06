use dioxus::prelude::*;

#[inline_props]
pub fn Consoles(
    cx: Scope,
    port_buffer: UseRef<Vec<String>>,
    user_buffer: UseRef<Vec<String>>,
) -> Element {
    render! {
        div {
            class: "flex flex-row gap-2 flex-1",
            Console { id: 0, buffer: user_buffer.clone() }
            Console { id: 1, buffer: port_buffer.clone() }
        }
    }
}

#[inline_props]
fn Console(cx: Scope, id: usize, buffer: UseRef<Vec<String>>) -> Element {
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

    let content = buffer.read().iter().cloned().collect::<String>();

    render! {
        div {
            class: "relative flex-1",
            textarea {
                id: "{element_id}",
                class: "textarea bg-base-200 w-full h-full text-sm",
                readonly: true,
                resize: "none",
                "{content}"
            }
            button {
                class: "btn btn-error absolute btn-outline text-sm",
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
