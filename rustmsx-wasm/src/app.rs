use std::rc::Rc;

use gloo::timers::callback::Interval;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    layout::{Memory, Navbar, Program, Registers, Screen, Vdp},
    store::{self, ComputerState, ExecutionState},
};

pub struct App {
    interval: Option<Interval>,
    state: Rc<ComputerState>,
    dispatch: Dispatch<ComputerState>,
}

pub enum Msg {
    State(Rc<ComputerState>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let on_change = ctx.link().callback(Msg::State);
        let dispatch = Dispatch::<ComputerState>::subscribe(on_change);

        Self {
            interval: None,
            state: dispatch.get(),
            dispatch,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                self.state = state;

                if self.state.state == ExecutionState::Running {
                    if self.interval.is_none() {
                        let dispatch = self.dispatch.clone();
                        let interval = Interval::new(1000 / 60, move || {
                            dispatch.apply(store::Msg::Tick);
                        });
                        self.interval = Some(interval);
                    }
                } else if let Some(interval) = self.interval.take() {
                    tracing::debug!("Stopping interval");
                    interval.forget();
                    self.interval = None;
                } else {
                    tracing::debug!("Interval already stopped");
                }

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let msx = self.state.msx.borrow();
        let program = msx.program();
        let vram = msx.vram();
        let ram = msx.ram();
        let cpu = msx.cpu.clone();
        let vdp = msx.vdp();

        html! {
            <div id="root">
                <div class="container">
                    <Navbar />
                    <div class="main">
                        <Program data={program} pc={cpu.pc} />
                        <div class="status">
                            <Registers cpu={msx.cpu.clone()} vdp={vdp} />

                            <Screen />

                            <div class="split">
                                <Memory data={ram} />
                                <Vdp data={vram} />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
