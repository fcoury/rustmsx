mod components;
mod layout;
mod msx;

use std::{cell::RefCell, rc::Rc};

use gloo::file::File;
use yew::prelude::*;

use components::Hexdump;
use msx::Msx;

use crate::layout::Navbar;

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
fn Memory() -> Html {
    html! {
        <div class="memory">
            <Hexdump />
        </div>
    }
}

#[function_component]
fn App() -> Html {
    let msx = Rc::new(RefCell::new(Msx::new()));

    let handle_rom_upload = Callback::from(move |data: Vec<u8>| {
        let mut msx = msx.borrow_mut();
        msx.load_rom(&data).unwrap();
        tracing::info!("Loaded!");
    });

    html! {
        <div class="container">
            <Navbar on_rom_upload={handle_rom_upload} />
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
