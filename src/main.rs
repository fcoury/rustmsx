mod components;
mod layout;
mod msx;

use std::sync::{Arc, RwLock};

use gloo::timers::callback::Interval;
use layout::Vdp;
use yew::prelude::*;

use msx::Msx;

use crate::{
    layout::{Memory, Navbar, Program, Registers},
    msx::EventType,
};

struct App {
    msx: Arc<RwLock<Msx>>,
    interval: Option<Interval>,
}

enum Msg {
    RomLoaded(Vec<u8>),
    Step,
    Start,
    Pause,
    Tick,
    Refresh,
    MsxEvent(EventType),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut msx = Msx::new();
        let link = ctx.link().clone();
        msx.subscribe(Box::new(move |event| {
            link.send_message(Msg::MsxEvent(event));
        }));

        let msx = Arc::new(RwLock::new(msx));

        Self {
            msx,
            interval: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RomLoaded(data) => {
                let mut msx = self.msx.write().unwrap();
                msx.load_rom(&data).unwrap();
                true
            }
            Msg::Refresh => {
                let msx = self.msx.read().unwrap();
                tracing::info!("MSX: {:#?}", msx.cpu);
                true
            }
            Msg::Step => {
                let mut msx = self.msx.write().unwrap();
                msx.step();
                true
            }
            Msg::Start => {
                let handle = {
                    let link = ctx.link().clone();
                    Interval::new(1, move || {
                        link.send_message(Msg::Tick);
                    })
                };
                self.interval = Some(handle);
                true
            }
            Msg::Pause => {
                self.interval.take().unwrap().cancel();
                true
            }
            Msg::Tick => {
                let mut msx = self.msx.write().unwrap();
                for _ in 0..1000 {
                    msx.step();
                }
                true
            }
            Msg::MsxEvent(event) => {
                tracing::debug!("Other: {:?}", event);
                false
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
        let handle_refresh = Callback::from(move |_| {
            link.send_message(Msg::Refresh);
        });

        let link = ctx.link().clone();
        let handle_step = Callback::from(move |_| {
            link.send_message(Msg::Step);
        });

        let link = ctx.link().clone();
        let has_interval = self.interval.is_some();
        let handle_run = Callback::from(move |_| {
            if has_interval {
                link.send_message(Msg::Pause);
                return;
            }
            link.send_message(Msg::Start);
        });

        html! {
            <div id="root">
                <div class="container">
                    <Navbar on_rom_upload={handle_rom_upload} on_refresh={handle_refresh} on_step={handle_step} on_run={handle_run} />
                    <div class="main">
                        <Program data={program} pc={&cpu.pc} />
                        <div class="status">
                            <Registers cpu={cpu.clone()} />
                            <div class="split">
                                <Memory data={cpu.memory.data} />
                                <Vdp data={msx.vdp.vram.to_vec()} />
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
