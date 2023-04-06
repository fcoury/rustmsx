use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::{
    layout::{Memory, Navbar, Program, Registers, Vdp},
    store::Store,
};

#[function_component]
pub fn App() -> Html {
    let (store, _) = use_store::<Store>();

    let msx = store.msx.borrow();
    let program = msx.program();
    let vram = msx.vram();
    let ram = msx.ram();
    let cpu = msx.cpu.clone();

    html! {
        <div id="root">
            <div class="container">
                <Navbar />
                <div class="main">
                    <Program data={program} pc={cpu.pc} />
                    <div class="status">
                        <Registers cpu={msx.cpu.clone()} />

                        // <Screen screen_buffer={vram.clone()} />

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
