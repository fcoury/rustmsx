use yew::prelude::*;

use crate::msx::components::cpu::Z80;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub cpu: Z80,
}

#[function_component]
pub fn Registers(props: &Props) -> Html {
    let cpu = &props.cpu;
    html! {
        <div class="registers">
            <div class="register">
                <div class="register__name">{ "PC" }</div>
                <div class="register__value">{ format!("{:04X}", cpu.pc) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "A" }</div>
                <div class="register__value">{ format!("{:02X}", cpu.a ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "B" }</div>
                <div class="register__value">{ format!("{:02X}", cpu.b ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "C" }</div>
                <div class="register__value">{ format!("{:02X}", cpu.c ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "D" }</div>
                <div class="register__value">{ format!("{:02X}", cpu.d ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "E" }</div>
                <div class="register__value">{ format!("{:02X}", cpu.d ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "F" }</div>
                <div class="register__value">{ format!("{:02X}", cpu.f ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "SP" }</div>
                <div class="register__value">{ format!("{:04X}", cpu.sp ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "HL" }</div>
                <div class="register__value">{ format!("{:04X}", cpu.get_hl() ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "AF" }</div>
                <div class="register__value">{ format!("{:04X}", cpu.get_af() ) }</div>
            </div>
            <div class="register">
                <div class="register__name">{ "BC" }</div>
                <div class="register__value">{ format!("{:04X}", cpu.get_bc() ) }</div>
            </div>
        </div>
    }
}
