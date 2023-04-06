use yew::prelude::*;

use crate::components::Hexdump;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub data: Vec<u8>,
}

#[function_component]
pub fn Memory(props: &Props) -> Html {
    html! {
        <div class="memory">
            <Hexdump data={props.clone().data} columns={8} />
        </div>
    }
}
