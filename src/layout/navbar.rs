use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    components::FileUploadButton,
    store::{Msg, Store},
};

#[function_component]
pub fn Navbar() -> Html {
    let (state, dispatch) = use_store::<Store>();

    let d = dispatch.clone();
    let on_rom_upload = Callback::from(move |rom: Vec<u8>| d.apply(Msg::LoadRom(rom)));

    let d = dispatch.clone();
    let handle_step_click = Callback::from(move |_| d.apply(Msg::Step));

    let d = dispatch;
    let handle_run_click = Callback::from(move |_| d.apply(Msg::Toggle));

    let label = match state.state {
        crate::store::ComputerState::Off => "Run",
        crate::store::ComputerState::Started => "Pause",
        crate::store::ComputerState::Paused => "Run",
    };

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
            <div class="navbar__item">
                <button onclick={handle_run_click}>{ label }</button>
            </div>
        </div>
    }
}
