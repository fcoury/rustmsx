use std::collections::HashMap;

use gloo::file::{callbacks::FileReader, File};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

pub struct FileUploadButton {
    readers: HashMap<String, FileReader>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_upload: Callback<Vec<u8>>,
    pub children: Children,
}

pub enum Msg {
    File(File),
    Uploaded(Vec<u8>),
}

impl Component for FileUploadButton {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::File(file) => {
                let link = ctx.link().clone();
                let task = gloo::file::callbacks::read_as_bytes(&file, move |res| {
                    link.send_message(Msg::Uploaded(res.unwrap()));
                });
                self.readers.insert(file.name(), task);

                true
            }
            Msg::Uploaded(data) => {
                ctx.props().on_upload.emit(data);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_open_rom = {
            let link = ctx.link().clone();
            Callback::from(move |_| {
                let link = link.clone();
                let on_change_closure = Closure::wrap(Box::new(move |event: Event| {
                    let input = event
                        .target()
                        .unwrap()
                        .dyn_into::<HtmlInputElement>()
                        .unwrap();
                    if let Some(file_list) = input.files() {
                        let mut files = js_sys::try_iter(&file_list)
                            .unwrap()
                            .unwrap()
                            .map(|file| web_sys::File::from(file.unwrap()))
                            .map(File::from);
                        link.send_message(Msg::File(files.next().unwrap()));
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
