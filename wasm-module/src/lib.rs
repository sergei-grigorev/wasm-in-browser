#![no_std]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use arrow::{ipc::reader::StreamReader, record_batch::RecordBatch};
use js_sys::{ArrayBuffer, Uint8Array};
use thiserror_no_std::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Error, Debug)]
enum AggregateError {
    #[error("Data was not received")]
    RequestFailed(JsValue),
    #[error("Cannot decode RecordBatch")]
    DecodingError,
    #[error("Data has a wrong format")]
    CastError,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}

#[wasm_bindgen]
pub async fn run_aggregation() -> Result<u32, JsValue> {
    match run_aggregation_internal().await {
        Ok(res) => Ok(res),
        Err(AggregateError::RequestFailed(err)) => Err(err),
        Err(err) => Err(JsValue::from_str(&format!("Aggregation failed: {err}"))),
    }
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> Result<u32, JsValue> {
    Ok(a + b)
}

#[wasm_bindgen]
pub fn say_hi(user: String) -> String {
    format!("Hello, {}", user)
}

async fn run_aggregation_internal() -> Result<u32, AggregateError> {
    let data = request_data()
        .await
        .map_err(AggregateError::RequestFailed)?;

    let batch: RecordBatch = {
        let mut stream_reader = StreamReader::try_new(data.as_slice(), None)
            .map_err(|_| AggregateError::DecodingError)?;
        if let Some(elem) = stream_reader.next() {
            elem.map_err(|_| AggregateError::DecodingError)?
        } else {
            return Err(AggregateError::DecodingError);
        }
    };

    Ok(batch.num_rows() as u32)
}

async fn request_data() -> Result<Vec<u8>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::SameOrigin);

    let url = "/data";

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/octet-stream")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let body = JsFuture::from(resp.array_buffer()?).await?;
    let array: Uint8Array = Uint8Array::new(&body);
    let vec: Vec<_> = array.to_vec();

    Ok(vec)
}
