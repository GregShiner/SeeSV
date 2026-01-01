#![allow(dead_code)]

use core::slice;
use std::ffi::NulError;

#[repr(C)]
pub enum ExternalDataType {
    Int,
    Float,
    String,
}

#[repr(C)]
struct ExternalTableView {
    columns: *const ExternalColumnView,
    num_of_columns: usize,
}

#[repr(C)]
struct ExternalColumnView {
    name: *const u8,
    name_len: usize,
    chunks: *const ExternalChunkView,
    num_of_chunks: usize,
    data_type: ExternalDataType,
}

#[repr(C)]
struct ExternalChunkView {
    sub_chunks: *const ExternalSubChunkView,
    referenced_chunks: *const usize,
    num_of_sub_chunks: usize,
}

#[repr(C)]
struct ExternalSubChunkView {
    data: *const u8,
    offsets: *const usize,
    lengths: *const usize,
    num_of_items: usize,
}

pub struct TableView<'a> {
    pub columns: Vec<ColumnView<'a>>,
}

pub struct ColumnView<'a> {
    pub name: &'a str,
    pub chunks: ChunkViews<'a>,
}

pub enum ChunkViews<'a> {
    Int(Vec<Chunk<'a, IntValues<'a>>>),
    Float(Vec<Chunk<'a, FloatValues<'a>>>),
    String(Vec<Chunk<'a, StringValues<'a>>>),
}

pub struct Chunk<'a, T> {
    pub sub_chunks: Vec<SubChunk<'a, T>>,
}

pub struct SubChunk<'a, T> {
    pub values: T,
    _phantom: std::marker::PhantomData<&'a ()>,
}

pub type IntValues<'a> = &'a [i32];
pub type FloatValues<'a> = &'a [f32];
pub type StringValues<'a> = Vec<&'a str>;

trait FromExternalSubChunk<'a>: Sized {
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> Self;
}

trait FromExternalChunk<'a> {
    unsafe fn from_external(chunk: &'a ExternalChunkView) -> Self;
}

impl<'a> FromExternalSubChunk<'a> for SubChunk<'a, IntValues<'a>> {
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> Self {
        unsafe {
            SubChunk {
                values: slice::from_raw_parts(sub_chunk.data as *const i32, sub_chunk.num_of_items),
                _phantom: std::marker::PhantomData,
            }
        }
    }
}

impl<'a> FromExternalSubChunk<'a> for SubChunk<'a, FloatValues<'a>> {
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> Self {
        unsafe {
            SubChunk {
                values: slice::from_raw_parts(sub_chunk.data as *const f32, sub_chunk.num_of_items),
                _phantom: std::marker::PhantomData,
            }
        }
    }
}

impl<'a> FromExternalSubChunk<'a> for SubChunk<'a, StringValues<'a>> {
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> Self {
        let mut values = Vec::with_capacity(sub_chunk.num_of_items);
        unsafe {
            let offsets = slice::from_raw_parts(sub_chunk.offsets, sub_chunk.num_of_items);
            let lengths = slice::from_raw_parts(sub_chunk.lengths, sub_chunk.num_of_items);
            for i in 0..sub_chunk.num_of_items {
                values.push(str::from_utf8_unchecked(slice::from_raw_parts(
                    sub_chunk.data.wrapping_add(offsets[i]),
                    lengths[i],
                )));
            }
        }
        SubChunk {
            values,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> FromExternalChunk<'a> for Chunk<'a, T>
where
    SubChunk<'a, T>: FromExternalSubChunk<'a>,
{
    unsafe fn from_external(chunk: &'a ExternalChunkView) -> Chunk<'a, T> {
        // let mut sub_chunks = Vec::with_capacity(chunk.num_of_sub_chunks)
        unsafe {
            let external_sub_chunks =
                slice::from_raw_parts(chunk.sub_chunks, chunk.num_of_sub_chunks);
            let sub_chunks: Vec<_> = external_sub_chunks
                .iter()
                .map(|external_sub_chunk| SubChunk::from_external(external_sub_chunk))
                .collect();
            Chunk { sub_chunks }
        }
    }
}

impl<'a> ColumnView<'a> {
    unsafe fn from_external(column: &'a ExternalColumnView) -> ColumnView<'a> {
        unsafe {
            let name =
                str::from_utf8_unchecked(slice::from_raw_parts(column.name, column.name_len));
            let external_chunks = slice::from_raw_parts(column.chunks, column.num_of_chunks);

            #[rustfmt::skip]
            let chunks = match column.data_type {
                ExternalDataType::Int => ChunkViews::Int(
                    Self::convert_chunks::<IntValues>(external_chunks)
                ),
                ExternalDataType::Float => ChunkViews::Float(
                    Self::convert_chunks::<FloatValues>(external_chunks)
                ),
                ExternalDataType::String => ChunkViews::String(
                    Self::convert_chunks::<StringValues>(external_chunks)
                ),
            };
            ColumnView { name, chunks }
        }
    }

    unsafe fn convert_chunks<T>(external_chunks: &'a [ExternalChunkView]) -> Vec<Chunk<'a, T>>
    where
        Chunk<'a, T>: FromExternalChunk<'a>,
    {
        unsafe {
            external_chunks
                .iter()
                .map(|external_chunk| Chunk::from_external(external_chunk))
                .collect()
        }
    }
}

impl<'a> TableView<'a> {
    fn from_external(table: ExternalTableView) -> TableView<'a> {
        unsafe {
            let external_columns = slice::from_raw_parts(table.columns, table.num_of_columns);
            TableView {
                columns: external_columns
                    .iter()
                    .map(|external_column| ColumnView::from_external(external_column))
                    .collect(),
            }
        }
    }
}

unsafe extern "C" {
    fn parse_csv(file_name: *const libc::c_char) -> ExternalTableView;
}

pub fn load_csv<'a>(file_name: &str) -> Result<TableView<'a>, NulError> {
    let c_filename = std::ffi::CString::new(file_name)?;
    let external = unsafe { parse_csv(c_filename.as_ptr()) };
    // TODO: Add error checks
    Ok(TableView::from_external(external))
}
