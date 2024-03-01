use leptos::*;
use service::SearchResult;
use thaw::*;
mod service;

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let search_query = create_rw_signal(String::new());

    let (actual_query, set_actual_query) = create_signal(String::new());

    let async_data = create_resource(
        move || actual_query.get(),
        |query| async move { service::search(query).await },
    );

    let search = move |_| {
        set_actual_query.set(search_query.get());
    };

    view! {
        <Layout>
            <h1>"Hello, world!"</h1>
            <Input value=search_query />

            <Button variant=ButtonVariant::Primary on:click=search >
                "Search"
            </Button>

            {move || match async_data.get() {
                None => view! { <p>"Loading..."</p> }.into_view(),
                Some(Ok(data)) => view! { <FileList data=data.clone() /> }.into_view(),
                Some(Err(e)) => view! { <p>{format!("Error: {}", e)}</p> }.into_view(),
            }}
        </Layout>
    }
}

#[component]
fn FileList(data: Vec<SearchResult>) -> impl IntoView {
    data.iter().map(|file| {
        view! {
            <div>
                <p>{file.path.clone()}</p>
                <p>{file.score}</p>
            </div>
        }
    }).collect_view()
}
