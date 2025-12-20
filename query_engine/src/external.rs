#[repr(C)]
pub enum DataType {
    INT,
    FLOAT,
    STRING,
}

#[repr(C)]
struct ExternalTableView {
    columns: *mut ExternalColumnView,
    num_of_columns: usize,
}

#[repr(C)]
struct ExternalColumnView {
    name: *mut u8,
    name_len: usize,
    chunks: *mut ExternalChunkView,
    num_of_chunks: usize,
    data_type: DataType,
}

#[repr(C)]
struct ExternalChunkView {
    sub_chunks: *mut *mut u8,
    referenced_chunks: *mut usize,
    sub_chunk_lens: *mut usize,
    num_of_sub_chunks: usize,
}

struct TableView<'a> {
    columns: &'a [ColumnView<'a>],
}

pub enum ColumnView<'a> {
    Int(IntColumn<'a>),
    Float(FloatColumn<'a>),
    String(StringColumn<'a>),
}

pub struct IntColumn<'a> {
    pub name: &'a str,
    pub chunks: &'a [IntChunk<'a>],
}

pub struct IntChunk<'a> {
    pub values: &'a [i32],
}

pub struct FloatColumn<'a> {
    pub name: &'a str,
    pub chunks: &'a [FloatChunk<'a>],
}

pub struct FloatChunk<'a> {
    pub values: &'a [f32],
}

pub struct StringColumn<'a> {
    pub name: &'a str,
    pub chunks: &'a [StringChunk<'a>],
}

pub struct StringChunk<'a> {
    pub values: &'a str,
}

impl From<ExternalChunkView> for StringChunk {
    unsafe fn from(value: ExternalChunkView) -> Self {
        IntChunk {
            values: core::slice::from_raw_parts(data, len),
        }
    }
}

unsafe extern "C" {
    fn parse_csv(file_name: *const std::ffi::c_char) -> ExternalTableView;
}
