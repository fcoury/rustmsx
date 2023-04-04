use yew::prelude::*;

use crate::msx::components::cpu::Z80;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub cpu: Z80,
}

#[function_component]
pub fn Registers(props: &Props) -> Html {
    html! {
        <div class="registers">
            <div class="register">
                <div class="register__name">{ "PC" }</div>
                <div class="register__value">{ props.cpu.pc }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "A" }</div>
                <div class="register__value">{ "F0" }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "B" }</div>
                <div class="register__value">{ "00" }</div>
            </div>
        </div>
    }
}
