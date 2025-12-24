#[repr(C)]
pub enum DataType {
    INT,
    FLOAT,
    STRING,
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
    data_type: DataType,
}

#[repr(C)]
struct ExternalChunkView {
    sub_chunks: *const ExternalSubChunkView,
    referenced_chunks: *const usize,
    sub_chunk_lens: *const usize,
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
    columns: &'a [ColumnView<'a>],
}

pub enum ChunkViews<'a> {
    Int(&'a [IntChunk<'a>]),
    Float(&'a [FloatChunk<'a>]),
    String(&'a [StringChunk<'a>]),
}
pub struct ColumnView<'a> {
    pub name: &'a str,
    pub chunks: ChunkViews<'a>,
}

pub struct IntChunk<'a> {
    pub sub_chunks: &'a [IntSubChunk<'a>],
}

pub struct FloatChunk<'a> {
    pub sub_chunks: &'a [FloatSubChunk<'a>],
}

pub struct StringChunk<'a> {
    pub sub_chunks: &'a [StringSubChunk<'a>],
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

impl<'a> IntSubChunk<'a> {
    unsafe fn from_external(sub_chunk: ExternalSubChunkView) -> IntSubChunk<'a> {
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

unsafe extern "C" {
    fn parse_csv(file_name: *const std::ffi::c_char) -> ExternalTableView;
}
