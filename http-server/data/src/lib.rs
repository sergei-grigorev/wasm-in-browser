use std::sync::Arc;

use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

use arrow::array::Int32Array;
use arrow::datatypes::{Field, SchemaBuilder};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

/// A simple Spin HTTP component.
#[http_component]
fn handle_data(req: Request) -> anyhow::Result<impl IntoResponse> {
    // array to be aggregated
    let column1 = Int32Array::from(vec![10, 20, 30]);
    let column2 = Int32Array::from(vec![30, 20, 10]);

    let mut schema = SchemaBuilder::with_capacity(2);
    schema.push(Field::new(
        "column1",
        arrow::datatypes::DataType::Int32,
        false,
    ));
    schema.push(Field::new(
        "column2",
        arrow::datatypes::DataType::Int32,
        false,
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
