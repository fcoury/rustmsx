use yew::prelude::*;

use crate::msx::ProgramEntry;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub data: Vec<ProgramEntry>,
}

#[function_component]
pub fn Program(props: &Props) -> Html {
    html! {
        <div class="opcodes">
            {
                props.data.iter().map(|entry| {
                    html! {
                        <div class="opcode">
                            <div class="opcode__column opcode__address">{ &entry.address }</div>
                            <div class="opcode__column opcode__hex">{ &entry.data }</div>
                            <div class="opcode__column opcode__instruction">
                                { &entry.instruction }
                            </div>
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}
