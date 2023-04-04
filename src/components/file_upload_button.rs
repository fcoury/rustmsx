use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{Event, File, HtmlInputElement};
use yew::prelude::*;

pub struct FileUploadButton;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_upload: Callback<File>,
    pub children: Children,
}

impl Component for FileUploadButton {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_open_rom = {
            let on_upload = Rc::new(ctx.props().on_upload.clone());
            Callback::from(move |_| {
                let on_upload = on_upload.clone();
                let on_change_closure = Closure::wrap(Box::new(move |event: Event| {
                    let input = event
                        .target()
                        .unwrap()
                        .dyn_into::<HtmlInputElement>()
                        .unwrap();
                    if let Some(file_list) = input.files() {
                        if let Some(file) = file_list.item(0) {
                            on_upload.emit(file);
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                let input = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("input")
                    .unwrap();
                input.set_attribute("type", "file").unwrap();
                input.set_attribute("accept", ".rom").unwrap();
                input.set_attribute("style", "display: none").unwrap();
                input.set_attribute("id", "file-input").unwrap();
                input
                    .add_event_listener_with_callback(
                        "change",
                        on_change_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                on_change_closure.forget();
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .append_child(&input)
                    .unwrap();
                let input = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("file-input")
                    .unwrap();
                input.dyn_ref::<HtmlInputElement>().unwrap().click();
            })
        };

        html! {
            <button onclick={on_open_rom}>{ for ctx.props().children.iter() }</button>
        }
    }
}
