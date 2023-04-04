use yew::prelude::*;

use crate::components::FileUploadButton;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_rom_upload: Callback<Vec<u8>>,
    pub on_step: Callback<()>,
}

#[function_component]
pub fn Navbar(props: &Props) -> Html {
    let on_rom_upload = props.on_rom_upload.clone();
    let props = props.clone();
    let handle_step_click = Callback::from(move |_| {
        props.on_step.emit(());
    });

    html! {
        <div class="navbar">
            <div class="navbar__item">
                <FileUploadButton on_upload={on_rom_upload}>{ "Open ROM" }</FileUploadButton>
            </div>
            <div class="navbar__item">
                <button>{ "Refresh" }</button>
            </div>
            <div class="navbar__item">
                <button onclick={handle_step_click}>{ "Step" }</button>
            </div>
        </div>
    }
}
