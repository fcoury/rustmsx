use std::rc::Rc;
use std::sync::{Arc, RwLock};

use gloo::timers::callback::Timeout;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::msx::Msx;
use crate::store::Store;
use crate::{
    layout::{Memory, Navbar, Program, Registers, Renderer, Screen, Vdp},
    msx::EventType,
};

pub struct App {
    store: Rc<Store>,
    dispatch: Dispatch<Store>,
}

pub enum Msg {
    RomLoaded(Vec<u8>),
    Step,
    Start,
    Pause,
    Tick,
    Refresh,
    MsxEvent(EventType),
    UpdateStore(Rc<Store>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::UpdateStore);
        let dispatch = Dispatch::<Store>::subscribe(callback);

        let mut msx = Msx::new();
        let link = ctx.link().clone();
        msx.subscribe(Box::new(move |event| {
            link.send_message(Msg::MsxEvent(event));
        }));

        let msx = Arc::new(RwLock::new(msx));

        Self {
            // msx,
            // timeout: None,
            // screen_buffer: [0; 256 * 192],
            store: dispatch.get(),
            dispatch,
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
                    Timeout::new(0, move || {
                        link.send_message(Msg::Tick);
                    })
                };
                self.timeout = Some(handle);
                true
            }
            Msg::Pause => {
                self.timeout.take().unwrap().cancel();
                true
            }
            Msg::Tick => {
                let mut msx = self.msx.write().unwrap();
                for _ in 0..5000 {
                    msx.step();
                }
                drop(msx);

                let msx = self.msx.read().unwrap();
                let vdp = msx.get_vdp();
                let mut renderer = Renderer::new(&vdp);
                renderer.draw(0, 0, 256, 192);
                self.screen_buffer = renderer.screen_buffer;

                let link = ctx.link().clone();
                self.timeout = Some(Timeout::new(0, move || {
                    link.send_message(Msg::Tick);
                }));
                true
            }
            Msg::MsxEvent(event) => {
                tracing::debug!("Other: {:?}", event);
                false
            }
            Msg::UpdateStore(store) => {}
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
        let has_interval = self.timeout.is_some();
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

                            <Screen screen_buffer={msx.vram()} />

                            <div class="split">
                                <Memory data={cpu.memory.data} />
                                <Vdp data={msx.vram()} />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
