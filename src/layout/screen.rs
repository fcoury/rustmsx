use std::rc::Rc;

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::store::ComputerState;

pub enum Msg {
    State(Rc<ComputerState>),
}

pub struct Screen {
    canvas_ref: NodeRef,
    state: Rc<ComputerState>,
    dispatch: Dispatch<ComputerState>,
}

impl Component for Screen {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let on_change = ctx.link().callback(Msg::State);
        let dispatch = Dispatch::<ComputerState>::subscribe(on_change);

        Self {
            canvas_ref: NodeRef::default(),
            state: dispatch.get(),
            dispatch,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                self.update_screen(state.screen_buffer.clone());
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
    fn update_screen(&mut self, screen_buffer: Vec<u8>) {
        if screen_buffer.len() < 256 * 192 {
            return;
        }

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
                let color = screen_buffer[color_offset];
                let mut color_bytes = palette[color as usize].to_le_bytes();
                color_bytes[3] = 255;
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
