// use tracing_subscriber::fmt::format::{FmtSpan, Pretty};
// use tracing_subscriber::fmt::time::UtcTime;
// use tracing_subscriber::prelude::*;
// use tracing_web::{performance_layer, MakeConsoleWriter};

use app::App;
use tracing_wasm::WASMLayerConfigBuilder;

mod app;
mod components;
mod layout;
mod store;

fn main() {
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_ansi(false) // Only partially supported across browsers
    //     .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
    //     .with_writer(MakeConsoleWriter)
    //     .with_span_events(FmtSpan::ACTIVE); // write events to the console
    // let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    // tracing_subscriber::registry()
    //     .with(fmt_layer)
    //     .with(perf_layer)
    //     .init(); // Install these as subscribers to tracing events

    yew::Renderer::<App>::new().render();
}
