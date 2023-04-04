use yew::prelude::*;

use crate::components::FileUploadButton;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_rom_upload: Callback<Vec<u8>>,
}

#[function_component]
pub fn Navbar(props: &Props) -> Html {
    let on_rom_upload = props.on_rom_upload.clone();
    // let on_upload = Callback::from(|file: File| {
    //     tracing::debug!("File: {:?}", file);
    // });

    html! {
        <div class="navbar">
            <div class="navbar__item">
                <FileUploadButton on_upload={on_rom_upload}>{ "Open ROM" }</FileUploadButton>
            </div>
            <div class="navbar__item">
                <button>{ "Refresh" }</button>
            </div>
            <div class="navbar__item">
                <button>{ "Step" }</button>
            </div>
        </div>
    }
}
