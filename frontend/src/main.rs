use leptos::*;
use thaw::*;
mod service;

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let async_data = create_resource(|| (), |_| async move { service::search("Anakin").await });

    view! {
        <div>
            <h1>"Hello, world!"</h1>
            {move || match async_data.get() {
        None => view! { <p>"Loading..."</p> }.into_view(),
        Some(Ok(data)) => view! { <p>"Loaded!"</p> }.into_view(),
        Some(Err(e)) => view! { <p>{format!("Error: {}", e)}</p> }.into_view(),
    }}
        </div>
    }
}
