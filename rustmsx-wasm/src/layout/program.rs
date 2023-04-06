use msx::ProgramEntry;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub data: Vec<ProgramEntry>,
    pub pc: u16,
}

#[function_component]
pub fn Program(props: &Props) -> Html {
    html! {
        <div class="opcodes">
            {
                props.data.iter().map(|entry| {
                    let mut classes = vec!["opcode"];
                    if entry.address == props.pc {
                        classes.push("opcode--current");
                    }
                    html! {
                        <div class={classes!(classes)}>
                            <div class="opcode__column opcode__address">{ format!("{:04X}", &entry.address) }</div>
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
