#![no_std]

extern crate alloc;

use alloc::{format, vec::Vec};
use arrow::array::Int32Array;
use arrow::compute::{max, min};
use arrow::{ipc::reader::StreamReader, record_batch::RecordBatch};
use js_sys::{Promise, Uint8Array};
use thiserror_no_std::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Error, Debug)]
pub enum AggregateError {
    #[error("Data was not received")]
    RequestFailed(JsValue),
    #[error("Cannot decode RecordBatch: {0}")]
    DecodingError(arrow::error::ArrowError),
    #[error("Unknown decoding error")]
    UnknownDecodingError,
    #[error("Data has a wrong format")]
    CastError,
}

impl Into<JsValue> for AggregateError {
    fn into(self) -> JsValue {
        JsValue::from_str(&format!("Aggregation failed: {self}"))
    }
}

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = console)]
    // fn log(s: &str);

    #[wasm_bindgen(js_name = fetch)]
    fn fetch_with_request(input: &Request) -> Promise;
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum AggregateMethod {
    MinSum,
    MaxSum,
}

#[wasm_bindgen]
pub struct AggregateTask {
    method: AggregateMethod,
}

#[wasm_bindgen]
impl AggregateTask {
    #[wasm_bindgen(constructor)]
    pub fn new(method: AggregateMethod) -> Self {
        Self { method }
    }
}

#[wasm_bindgen]
pub struct Dataset {
    internal: Option<RecordBatch>,
}

#[wasm_bindgen]
impl Dataset {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self { internal: None }
    }

    pub async fn fetch_data(&mut self) -> Result<usize, AggregateError> {
        let data = request_data()
            .await
            .map_err(AggregateError::RequestFailed)?;

        let batch: RecordBatch = {
            let mut stream_reader = StreamReader::try_new(data.as_slice(), None)
                .map_err(AggregateError::DecodingError)?;
            if let Some(elem) = stream_reader.next() {
                elem.map_err(AggregateError::DecodingError)?
            } else {
                return Err(AggregateError::UnknownDecodingError);
            }
        };

        let rows_count = batch.num_rows();
        self.internal = Some(batch);
        Ok(rows_count)
    }

    pub fn aggregate_data(&self, task: AggregateTask) -> Result<i32, AggregateError> {
        if let Some(batch) = self.internal.as_ref() {
            let column1: &Int32Array = batch
                .column(0)
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or(AggregateError::CastError)?;

            let column2: &Int32Array = batch
                .column(1)
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or(AggregateError::CastError)?;

            // combine both arrays and then retun the max element
            match task.method {
                AggregateMethod::MaxSum => {
                    let both: Int32Array = arrow::compute::binary(column1, column2, |a, b| a + b)
                        .map_err(|_| AggregateError::CastError)?;
                    Ok(max(&both).unwrap_or_default() as i32)
                }
                AggregateMethod::MinSum => {
                    let both: Int32Array =
                        arrow::compute::binary(column1, column2, |a, b| -(a + b))
                            .map_err(|_| AggregateError::CastError)?;
                    Ok(min(&both).unwrap_or_default() as i32)
                }
            }
        } else {
            Ok(0)
        }
    }
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

    let resp_value = JsFuture::from(fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let body = JsFuture::from(resp.array_buffer()?).await?;
    let array: Uint8Array = Uint8Array::new(&body);
    let vec: Vec<_> = array.to_vec();
    Ok(vec)
}
