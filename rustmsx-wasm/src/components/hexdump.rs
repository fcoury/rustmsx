use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or(8)]
    pub columns: u8,
    pub data: Vec<u8>,
}

#[function_component]
pub fn Hexdump(props: &Props) -> Html {
    let chunks = props.data.chunks(props.columns as usize);

    html! {
        <div class="hexdump">
            { for chunks.enumerate().map(|(index, chunk)| {
                let address = index * props.columns as usize;
                let hex_values = chunk.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>();
                let ascii_values = chunk.iter().map(|byte| {
                    if byte.is_ascii_graphic() { format!("{}", *byte as char) } else { ".".to_string() }
                }).collect::<Vec<_>>();

                html! {
                    <div class="hexdump__entry">
                        <div class="hexdump__address">{ format!("{:04X}", address) }</div>
                        <div class="hexdump__contents">{ hex_values.join(" ") }</div>
                        <div class="hexdump__contents">{ ascii_values.join("") }</div>
                    </div>
                }
            }) }
        </div>
    }
}
