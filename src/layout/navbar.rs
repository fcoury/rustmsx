use yew::prelude::*;

use crate::components::FileUploadButton;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub on_rom_upload: Callback<Vec<u8>>,
    pub on_refresh: Callback<()>,
    pub on_step: Callback<()>,
    pub on_run: Callback<()>,
}

#[function_component]
pub fn Navbar(props: &Props) -> Html {
    let on_rom_upload = props.on_rom_upload.clone();

    let on_refresh = props.on_refresh.clone();
    let handle_refresh = Callback::from(move |_| {
        on_refresh.emit(());
    });

    let props = props.clone();
    let handle_step_click = Callback::from(move |_| {
        props.on_step.emit(());
    });

    // let props = props.clone();
    let handle_run_click = Callback::from(move |_| {
        props.on_run.emit(());
    });

    html! {
        <div class="navbar">
            <div class="navbar__item">
                <FileUploadButton on_upload={on_rom_upload}>{ "Open ROM" }</FileUploadButton>
            </div>
            <div class="navbar__item">
                <button onclick={handle_refresh}>{ "Refresh" }</button>
            </div>
            <div class="navbar__item">
                <button onclick={handle_step_click}>{ "Step" }</button>
            </div>
            <div class="navbar__item">
                <button onclick={handle_run_click}>{ "Run" }</button>
            </div>
        </div>
    }
}
