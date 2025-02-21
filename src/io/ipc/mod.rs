//! APIs to read from and write to Arrow's IPC format.
//!
//! Inter-process communication is a method through which different processes
//! share and pass data between them. Its use-cases include parallel
//! processing of chunks of data across different CPU cores, transferring
//! data between different Apache Arrow implementations in other languages and
//! more. Under the hood Apache Arrow uses [FlatBuffers](https://google.github.io/flatbuffers/)
//! as its binary protocol, so every Arrow-centered streaming or serialiation
//! problem that could be solved using FlatBuffers could probably be solved
//! using the more integrated approach that is exposed in this module.
//!
//! [Arrow's IPC protocol](https://arrow.apache.org/docs/format/Columnar.html#serialization-and-interprocess-communication-ipc)
//! allows only batch or dictionary columns to be passed
//! around due to its reliance on a pre-defined data scheme. This constraint
//! provides a large performance gain because serialized data will always have a
//! known structutre, i.e. the same fields and datatypes, with the only variance
//! being the number of rows and the actual data inside the Batch. This dramatically
//! increases the deserialization rate, as the bytes in the file or stream are already
//! structured "correctly".
//!
//! Reading and writing IPC messages is done using one of two variants - either
//! [`FileReader`](read::FileReader) <-> [`FileWriter`](struct@write::FileWriter) or
//! [`StreamReader`](read::StreamReader) <-> [`StreamWriter`](struct@write::StreamWriter).
//! These two variants wrap a type `T` that implements [`Read`](std::io::Read), and in
//! the case of the `File` variant it also implements [`Seek`](std::io::Seek). In
//! practice it means that `File`s can be arbitrarily accessed while `Stream`s are only
//! read in certain order - the one they were written in (first in, first out).
//!
//! # Examples
//! Read and write to a file:
//! ```
//! use arrow2::io::ipc::{{read::{FileReader, read_file_metadata}}, {write::{FileWriter, WriteOptions}}};
//! # use std::fs::File;
//! # use std::sync::Arc;
//! # use arrow2::datatypes::{Field, Schema, DataType};
//! # use arrow2::array::{Int32Array, Array};
//! # use arrow2::chunk::Chunk;
//! # use arrow2::error::ArrowError;
//! // Setup the writer
//! let path = "example.arrow".to_string();
//! let mut file = File::create(&path)?;
//! let x_coord = Field::new("x", DataType::Int32, false);
//! let y_coord = Field::new("y", DataType::Int32, false);
//! let schema = Schema::from(vec![x_coord, y_coord]);
//! let options = WriteOptions {compression: None};
//! let mut writer = FileWriter::try_new(file, &schema, None, options)?;
//!
//! // Setup the data
//! let x_data = Int32Array::from_slice([-1i32, 1]);
//! let y_data = Int32Array::from_slice([1i32, -1]);
//! let chunk = Chunk::try_new(
//!     vec![Arc::new(x_data) as Arc<dyn Array>, Arc::new(y_data)]
//! )?;
//!
//! // Write the messages and finalize the stream
//! for _ in 0..5 {
//!     writer.write(&chunk, None);
//! }
//! writer.finish();
//!
//! // Fetch some of the data and get the reader back
//! let mut reader = File::open(&path)?;
//! let metadata = read_file_metadata(&mut reader)?;
//! let mut filereader = FileReader::new(reader, metadata, None);
//! let row1 = filereader.next().unwrap();  // [[-1, 1], [1, -1]]
//! let row2 = filereader.next().unwrap();  // [[-1, 1], [1, -1]]
//! let mut reader = filereader.into_inner();
//! // Do more stuff with the reader, like seeking ahead.
//! # Ok::<(), ArrowError>(())
//! ```
//!
//! For further information and examples please consult the
//! [user guide](https://jorgecarleitao.github.io/arrow2/io/index.html).
//! For even more examples check the `examples` folder in the main repository
//! ([1](https://github.com/jorgecarleitao/arrow2/blob/main/examples/ipc_file_read.rs),
//! [2](https://github.com/jorgecarleitao/arrow2/blob/main/examples/ipc_file_write.rs),
//! [3](https://github.com/jorgecarleitao/arrow2/tree/main/examples/ipc_pyarrow)).

mod compression;
mod endianess;

pub mod read;
pub mod write;

const ARROW_MAGIC: [u8; 6] = [b'A', b'R', b'R', b'O', b'W', b'1'];
const CONTINUATION_MARKER: [u8; 4] = [0xff; 4];

/// Struct containing `dictionary_id` and nested `IpcField`, allowing users
/// to specify the dictionary ids of the IPC fields when writing to IPC.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IpcField {
    // optional children
    pub fields: Vec<IpcField>,
    // dictionary id
    pub dictionary_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IpcSchema {
    pub fields: Vec<IpcField>,
    pub is_little_endian: bool,
}
