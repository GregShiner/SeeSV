#![allow(dead_code)]

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

struct TableView<'a> {
    columns: Vec<ColumnView<'a>>,
}

pub struct ColumnView<'a> {
    pub name: &'a str,
    pub chunks: ChunkViews<'a>,
}

pub enum ChunkViews<'a> {
    Int(Vec<IntChunk<'a>>),
    Float(Vec<FloatChunk<'a>>),
    String(Vec<StringChunk<'a>>),
}

pub struct IntChunk<'a> {
    pub sub_chunks: Vec<IntSubChunk<'a>>,
}

pub struct FloatChunk<'a> {
    pub sub_chunks: Vec<FloatSubChunk<'a>>,
}

pub struct StringChunk<'a> {
    pub sub_chunks: Vec<StringSubChunk<'a>>,
}

pub struct IntSubChunk<'a> {
    pub values: &'a [i32],
}

pub struct FloatSubChunk<'a> {
    pub values: &'a [f32],
}

pub struct StringSubChunk<'a> {
    pub values: Vec<&'a str>,
}

trait FromExternalSubChunk<'a> {
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> Self;
}

impl<'a> FromExternalSubChunk<'a> for IntSubChunk<'a> {
    /// # Safety
    /// - `sub_chunk.data` must be valid for `'a`
    /// - `sub_chunk.data` must be a valid array of aligned i32s
    /// - `num_of_items` must match number of f32's in array
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> IntSubChunk<'a> {
        unsafe {
            IntSubChunk {
                values: core::slice::from_raw_parts(
                    sub_chunk.data as *const i32,
                    sub_chunk.num_of_items,
                ),
            }
        }
    }
}

impl<'a> FromExternalSubChunk<'a> for FloatSubChunk<'a> {
    /// # Safety
    /// - `sub_chunk.data` must be valid for `'a`
    /// - `sub_chunk.data` must be a valid array of aligned f32s
    /// - `num_of_items` must match number of f32's in array
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> FloatSubChunk<'a> {
        unsafe {
            FloatSubChunk {
                values: core::slice::from_raw_parts(
                    sub_chunk.data as *const f32,
                    sub_chunk.num_of_items,
                ),
            }
        }
    }
}

impl<'a> FromExternalSubChunk<'a> for StringSubChunk<'a> {
    /// # Safety
    /// - `sub_chunk.data`, `offsets`, and `lengths` must be valid for `'a`
    /// - Offsets and lengths must describe valid UTF-8 slices within `data`
    /// - `num_of_items` must match offsets/lengths arrays
    unsafe fn from_external(sub_chunk: &'a ExternalSubChunkView) -> StringSubChunk<'a> {
        let mut values = Vec::with_capacity(sub_chunk.num_of_items);
        unsafe {
            let offsets = core::slice::from_raw_parts(sub_chunk.offsets, sub_chunk.num_of_items);
            let lengths = core::slice::from_raw_parts(sub_chunk.lengths, sub_chunk.num_of_items);
            for i in 0..sub_chunk.num_of_items {
                values.push(str::from_utf8_unchecked(core::slice::from_raw_parts(
                    sub_chunk.data.wrapping_add(offsets[i]),
                    lengths[i],
                )));
            }
        }
        StringSubChunk { values }
    }
}

trait FromExternalChunk<'a> {
    unsafe fn from_external(chunk: &'a ExternalChunkView) -> Self;
}

impl<'a> FromExternalChunk<'a> for IntChunk<'a> {
    unsafe fn from_external(chunk: &'a ExternalChunkView) -> IntChunk<'a> {
        // let mut sub_chunks = Vec::with_capacity(chunk.num_of_sub_chunks)
        unsafe {
            let external_sub_chunks =
                core::slice::from_raw_parts(chunk.sub_chunks, chunk.num_of_sub_chunks);
            let sub_chunks: Vec<_> = external_sub_chunks
                .iter()
                .map(|external_sub_chunk| IntSubChunk::from_external(external_sub_chunk))
                .collect();
            IntChunk { sub_chunks }
        }
    }
}

impl<'a> FromExternalChunk<'a> for FloatChunk<'a> {
    unsafe fn from_external(chunk: &'a ExternalChunkView) -> FloatChunk<'a> {
        // let mut sub_chunks = Vec::with_capacity(chunk.num_of_sub_chunks)
        unsafe {
            let external_sub_chunks =
                core::slice::from_raw_parts(chunk.sub_chunks, chunk.num_of_sub_chunks);
            let sub_chunks: Vec<_> = external_sub_chunks
                .iter()
                .map(|external_sub_chunk| FloatSubChunk::from_external(external_sub_chunk))
                .collect();
            FloatChunk { sub_chunks }
        }
    }
}

impl<'a> FromExternalChunk<'a> for StringChunk<'a> {
    unsafe fn from_external(chunk: &'a ExternalChunkView) -> StringChunk<'a> {
        // let mut sub_chunks = Vec::with_capacity(chunk.num_of_sub_chunks)
        unsafe {
            let external_sub_chunks =
                core::slice::from_raw_parts(chunk.sub_chunks, chunk.num_of_sub_chunks);
            let sub_chunks: Vec<_> = external_sub_chunks
                .iter()
                .map(|external_sub_chunk| StringSubChunk::from_external(external_sub_chunk))
                .collect();
            StringChunk { sub_chunks }
        }
    }
}

impl<'a> ColumnView<'a> {
    unsafe fn from_external(column: &'a ExternalColumnView) -> ColumnView<'a> {
        unsafe {
            let name =
                str::from_utf8_unchecked(core::slice::from_raw_parts(column.name, column.name_len));
            let external_chunks = core::slice::from_raw_parts(column.chunks, column.num_of_chunks);
            let chunks = match column.data_type {
                ExternalDataType::Int => ChunkViews::Int(
                    external_chunks
                        .iter()
                        .map(|external_chunk| IntChunk::from_external(external_chunk))
                        .collect(),
                ),
                ExternalDataType::Float => ChunkViews::Float(
                    external_chunks
                        .iter()
                        .map(|external_chunk| FloatChunk::from_external(external_chunk))
                        .collect(),
                ),
                ExternalDataType::String => ChunkViews::String(
                    external_chunks
                        .iter()
                        .map(|external_chunk| StringChunk::from_external(external_chunk))
                        .collect(),
                ),
            };
            ColumnView { name, chunks }
        }
    }
}

impl<'a> TableView<'a> {
    unsafe fn from_external(table: &'a ExternalTableView) -> TableView<'a> {
        unsafe {
            let external_columns = core::slice::from_raw_parts(table.columns, table.num_of_columns);
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
    fn parse_csv(file_name: *const std::ffi::c_char) -> ExternalTableView;
}
