use std::rc::Rc;

use msx::Msx;
use yewdux::{mrc::Mrc, prelude::*};

use crate::layout::Renderer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LoadRom(Vec<u8>),
    Toggle,
    Step,
    Tick,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum ExecutionState {
    #[default]
    Off,
    Running,
    Paused,
}

#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct ComputerState {
    pub msx: Mrc<Msx>,
    pub screen_buffer: Vec<u8>,
    pub state: ExecutionState,
    pub error: Option<String>,
}

impl Reducer<ComputerState> for Msg {
    fn apply(self, mut store: Rc<ComputerState>) -> Rc<ComputerState> {
        let state = Rc::make_mut(&mut store);

        // tracing::info!("[{:?}] Received message: {:?}", state.state, self);

        match self {
            Msg::Toggle => {
                state.state = match state.state {
                    ExecutionState::Off => ExecutionState::Running,
                    ExecutionState::Running => ExecutionState::Paused,
                    ExecutionState::Paused => ExecutionState::Running,
                };
            }
            Msg::Tick => {
                if state.state != ExecutionState::Running {
                    return store;
                }

                for _ in 0..10000 {
                    state.msx.borrow_mut().step();

                    if state.msx.borrow().current_scanline == 0 {
                        let msx = state.msx.borrow();
                        let vdp = msx.get_vdp();
                        let mut renderer = Renderer::new(&vdp);
                        renderer.draw(0, 0, 256, 192);
                        state.screen_buffer = renderer.screen_buffer.to_vec();
                    }

                    if state.state != ExecutionState::Running {
                        break;
                    }
                }
            }
            Msg::Step => {
                state.msx.borrow_mut().step();
            }
            // Msg::Render(new_buffer) => {
            //     state.screen_buffer = new_buffer;
            // }
            Msg::LoadRom(data) => {
                let mut msx = state.msx.borrow_mut();
                msx.load_rom(0, &data);
                msx.load_empty(1);
                msx.load_empty(2);
                msx.load_ram(3);
            }
        };

        store
    }
}
