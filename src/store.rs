use std::{rc::Rc, thread, time::Duration};

use yewdux::{mrc::Mrc, prelude::*};

use crate::msx::Msx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LoadRom(Vec<u8>),
    Toggle,
    Step,
    Tick,
    Render(Vec<u8>),
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

        tracing::info!("[{:?}] Received message: {:?}", state.state, self);

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

                for _ in 0..1000 {
                    state.msx.borrow_mut().step();
                    if state.state != ExecutionState::Running {
                        break;
                    }
                }
            }
            Msg::Step => {
                state.msx.borrow_mut().step();
            }
            Msg::Render(new_buffer) => {
                state.screen_buffer = new_buffer;
            }
            Msg::LoadRom(data) => {
                if let Err(e) = state.msx.borrow_mut().load_rom(&data) {
                    state.error = Some(e.to_string());
                }
            }
        };

        store
    }
}
