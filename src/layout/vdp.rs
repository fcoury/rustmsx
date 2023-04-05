use yew::prelude::*;

use crate::components::Hexdump;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub data: Vec<u8>,
}

#[function_component]
pub fn Vdp(props: &Props) -> Html {
    html! {
        <div class="vram">
            <Hexdump data={props.data.clone()} columns={8} />
        </div>
    }
}
