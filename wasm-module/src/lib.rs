#![no_std]

extern crate alloc;

use alloc::{format, vec::Vec};
use arrow::{ipc::reader::StreamReader, record_batch::RecordBatch};
use js_sys::{Promise, Uint8Array};
use polars::prelude::*;
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
    #[error("Data has a wrong format: {0}")]
    ConverError(PolarsError),
    #[error("Computation failed: {0}")]
    ComputeError(PolarsError),
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
            // transform to Polars
            let df: LazyFrame = {
                let schema = batch.schema();
                let size = batch.num_rows();
                let columns = batch.num_columns();
                let mut list = Vec::with_capacity(size);
                for i in 0..columns {
                    let name = schema.fields.get(i).unwrap().name();
                    let row = batch.column(i);
                    let series = Series::from_arrow_rs(name.as_str(), row.as_ref())
                        .map_err(AggregateError::ConverError)?;
                    list.push(series);
                }

                let df = DataFrame::new(list).map_err(AggregateError::ConverError)?;
                df.lazy()
            };

            // combine both arrays and then return the max element
            match task.method {
                AggregateMethod::MaxSum => {
                    let sum_column = col("column1") + col("column2");
                    let df: DataFrame = df
                        .filter(col("column1").gt(lit(0)))
                        .filter(col("column2").gt(lit(0)))
                        .select([sum_column.max()])
                        .collect()
                        .map_err(AggregateError::ComputeError)?;

                    if let Some(row) = df.get(0) {
                        if let AnyValue::Int32(res) = row.first().unwrap() {
                            Ok(*res)
                        } else {
                            unreachable!()
                        }
                    } else {
                        Ok(0)
                    }
                }
                AggregateMethod::MinSum => {
                    let sum_column = lit(0) - (col("column1") + col("column2"));
                    let df: DataFrame = df
                        .filter(col("column1").gt(lit(0)))
                        .filter(col("column2").gt(lit(0)))
                        .select([sum_column.min()])
                        .collect()
                        .map_err(AggregateError::ComputeError)?;

                    if let Some(row) = df.get(0) {
                        if let AnyValue::Int32(res) = row.first().unwrap() {
                            Ok(*res)
                        } else {
                            unreachable!()
                        }
                    } else {
                        Ok(0)
                    }
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
