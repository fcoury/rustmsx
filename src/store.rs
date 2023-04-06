use std::rc::Rc;

use yewdux::{mrc::Mrc, prelude::*};

use crate::msx::Msx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    LoadRom(Vec<u8>),
    Toggle,
    Step,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum ComputerState {
    #[default]
    Off,
    Started,
    Paused,
}

#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct Store {
    pub msx: Mrc<Msx>,
    pub state: ComputerState,
    pub error: Option<String>,
}

impl Reducer<Store> for Msg {
    fn apply(self, mut store: Rc<Store>) -> Rc<Store> {
        let state = Rc::make_mut(&mut store);

        match self {
            Msg::Toggle => {
                state.state = match state.state {
                    ComputerState::Off => ComputerState::Started,
                    ComputerState::Started => ComputerState::Paused,
                    ComputerState::Paused => ComputerState::Started,
                };
            }
            Msg::Step => {
                state.msx.borrow_mut().step();
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
