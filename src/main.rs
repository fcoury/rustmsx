use web_sys::File;
use yew::prelude::*;

use components::file_upload_button::FileUploadButton;

mod components;
mod msx;

#[function_component]
fn Navbar() -> Html {
    let on_upload = Callback::from(|file: File| {
        tracing::debug!("File: {:?}", file);
    });

    html! {
        <div class="navbar">
            <div class="navbar__item">
                <FileUploadButton on_upload={on_upload}>{ "Open ROM" }</FileUploadButton>
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

#[function_component]
fn Program() -> Html {
    html! {
        <div className="opcodes">
            <div class="opcode">
                <div class="opcode__column opcode__address">{ "0010" }</div>
                <div class="opcode__column opcode__hex">{ "AE 2D" }</div>
                <div class="opcode__column opcode__instruction">
                    { "ADD A, B" }
                </div>
            </div>
        </div>
    }
}

#[function_component]
fn Registers() -> Html {
    html! {
        <div class="registers">
            <div class="register">
                <div class="register__name">{ "A" }</div>
                <div class="register__value">{ "F0" }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "B" }</div>
                <div class="register__value">{ "00" }</div>
            </div>
        </div>
    }
}

#[function_component]
fn Hexdump() -> Html {
    html! {
        <div className="hexdump">
            <div className="hexdump__entry">
                <div className="hexdump__address">{ "0000" }</div>
                <div className="hexdump__contents"></div>
                <div className="hexdump__contents"></div>
            </div>
        </div>
    }
}

#[function_component]
fn Memory() -> Html {
    html! {
        <div class="memory">
            <Hexdump />
        </div>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <div class="container">
            <Navbar />
            <div class="main">
                <Program />
                <div class="status">
                    <Registers />
                    <div class="split">
                        <Memory />
                    </div>
                </div>
            </div>
        </div>
    }
}

fn main() {
    tracing_wasm::set_as_global_default();

    yew::Renderer::<App>::new().render();
}
