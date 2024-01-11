use std::sync::Arc;

use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

use arrow::array::Int32Builder;
use arrow::datatypes::{Field, SchemaBuilder};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

// 5,000,000
const RECORDS_COUNT: usize = 5000000;

/// A simple Spin HTTP component.
#[http_component]
fn handle_data(_: Request) -> anyhow::Result<impl IntoResponse> {
    // array to be aggregated
    let mut column1 = Int32Builder::with_capacity(RECORDS_COUNT);
    let mut column2 = Int32Builder::with_capacity(RECORDS_COUNT);

    for i in 0..RECORDS_COUNT {
        if i % 3 != 0 {
            column1.append_value(i as i32);
            column2.append_value(i as i32);
        } else {
            column1.append_null();
            column2.append_null();
        }
    }

    let column1 = column1.finish();
    let column2 = column2.finish();

    let mut schema = SchemaBuilder::with_capacity(2);
    schema.push(Field::new(
        "column1",
        arrow::datatypes::DataType::Int32,
        true,
    ));
    schema.push(Field::new(
        "column2",
        arrow::datatypes::DataType::Int32,
        true,
    ));
    let schema = schema.finish();

    let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(column1), Arc::new(column2)])?;

    // serialize to byte buffer
    let mut serialized: Vec<u8> = Vec::with_capacity(batch.get_array_memory_size() * 2);
    {
        let mut stream_writer = StreamWriter::try_new(&mut serialized, &batch.schema()).unwrap();
        stream_writer.write(&batch).unwrap();
    }

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/octet-stream")
        .header("Content-Length", serialized.len().to_string())
        .body(serialized)
        .build())
}
