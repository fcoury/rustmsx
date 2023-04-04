use yew::prelude::*;

#[function_component]
pub fn Hexdump() -> Html {
    html! {
        <div className="hexdump">
            <div className="hexdump__entry">
                <div className="hexdump__address">{ "0000" }</div>
                <div className="hexdump__contents"></div>
                <div className="hexdump__contents"></div>
            </div>
        </div>
    }
}
