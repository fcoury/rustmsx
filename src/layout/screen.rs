use gloo::timers::callback::Interval;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;

pub enum Msg {
    UpdateScreen,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub screen_buffer: Vec<u8>,
}

pub struct Screen {
    canvas_ref: NodeRef,
    screen_buffer: Vec<u8>,
    #[allow(dead_code)]
    interval: Option<Interval>,
}

impl Component for Screen {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            screen_buffer: vec![0; 256 * 192],
            interval: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateScreen => {
                self.update_screen();
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas ref={&self.canvas_ref} width="256" height="192"></canvas>
        }
    }
}

impl Screen {
    fn update_screen(&mut self) {
        let canvas: HtmlCanvasElement = self.canvas_ref.cast().unwrap();
        let ctx = canvas.get_context("2d").unwrap().unwrap();
        let ctx = ctx.dyn_into::<CanvasRenderingContext2d>().unwrap();

        let palette: [u32; 16] = [
            0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xAA5500, 0xAAAAAA,
            0x555555, 0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF,
        ];

        let width = 256;
        let height = 192;

        let mut data = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let color_offset = y * width + x;
                let color = self.screen_buffer[color_offset];
                let color_bytes = palette[color as usize].to_le_bytes();
                data.extend_from_slice(&color_bytes);
            }
        }

        let data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&data),
            width as u32,
            height as u32,
        )
        .unwrap();
        ctx.put_image_data(&data, 0.0, 0.0).unwrap();
    }
}
