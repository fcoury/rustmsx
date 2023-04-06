use app::App;

mod app;
mod components;
mod layout;
mod msx;
mod store;

fn main() {
    tracing_wasm::set_as_global_default();

    yew::Renderer::<App>::new().render();
}
