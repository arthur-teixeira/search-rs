use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub score: f32,
}

async fn do_request(method: &str, url: &str) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;

    let window = web_sys::window().unwrap();
    let promise = window.fetch_with_request(&request);
    let as_future = JsFuture::from(promise);
    match as_future.await {
        Ok(res) => {
            let res: Response = res.dyn_into().unwrap();
            if res.ok() {
                JsFuture::from(res.json()?).await
            } else {
                Err(JsValue::from_str("Request failed"))
            }
        }
        Err(err) => Err(err),
    }
}

const BASE_URL: &str = "http://localhost:3000";

pub async fn search(query: String) -> Result<Vec<SearchResult>, String> {
    let url = format!("{}/search?query={}", BASE_URL, query);

    if query == "" {
        return Ok(vec![]);
    }

    let result = match do_request("GET", &url).await {
        Ok(r) => r,
        Err(_) => return Err("Failed to fetch".to_string()),
    };

    match from_value(result) {
        Ok(r) => Ok(r),
        Err(e) => {
            web_sys::console::error_1(&JsValue::from(e));
            Err("Failed to parse response".to_string())
        }
    }
}
