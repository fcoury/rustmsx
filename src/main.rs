mod components;
mod layout;
mod msx;

use std::sync::{Arc, RwLock};

use yew::prelude::*;

use components::Hexdump;
use msx::Msx;

use crate::layout::{Navbar, Program, Registers};

#[function_component]
fn Memory() -> Html {
    html! {
        <div class="memory">
            <Hexdump />
        </div>
    }
}

struct App {
    msx: Arc<RwLock<Msx>>,
}

enum Msg {
    RomLoaded(Vec<u8>),
    Step,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            msx: Arc::new(RwLock::new(Msx::new())),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RomLoaded(data) => {
                let mut msx = self.msx.write().unwrap();
                msx.load_rom(&data).unwrap();
                true
            }
            Msg::Step => {
                let mut msx = self.msx.write().unwrap();
                msx.step();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let msx = self.msx.read().unwrap();
        let cpu = msx.cpu.clone();
        let program = msx.program();

        let link = ctx.link().clone();
        let handle_rom_upload = Callback::from(move |data: Vec<u8>| {
            link.send_message(Msg::RomLoaded(data));
            tracing::info!("Loaded!");
        });

        let link = ctx.link().clone();
        let handle_step = Callback::from(move |_| {
            link.send_message(Msg::Step);
        });

        html! {
            <div id="root">
                <div class="container">
                    <Navbar on_rom_upload={handle_rom_upload} on_step={handle_step} />
                    <div class="main">
                        <Program data={program} pc={cpu.pc} />
                        <div class="status">
                            <Registers cpu={cpu} />
                            <div class="split">
                                <Memory />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    tracing_wasm::set_as_global_default();

    yew::Renderer::<App>::new().render();
}
