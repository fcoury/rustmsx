use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod components;
mod msx;

#[function_component]
fn Navbar() -> Html {
    let on_open_rom = Callback::from(|_| {
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
            .set_attribute("onchange", "console.log(this.files)")
            .unwrap();
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
        input
            .dyn_ref::<web_sys::HtmlInputElement>()
            .unwrap()
            .click();
    });

    html! {
        <div class="navbar">
            <div class="navbar__item">
                <button onclick={on_open_rom.clone()}>{ "Open ROM" }</button>
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
    yew::Renderer::<App>::new().render();
}
