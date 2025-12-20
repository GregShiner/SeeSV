//! By convention, root.zig is the root source file when making a library.
const std = @import("std");

const DataType = enum {
    INT,
    FLOAT,
    STRING,
};

// const ZigDataDoNotTouch = struct {

const Chunk = struct {
    data: []u8,
    offsets: []usize,
    data_lens: []usize,

    // pub fn put(i: usize) []u8 {
    //     return data
    // }
};

const Column = struct {
    name: []u8,
    name_len: usize,
    chunks: []Chunk,
    num_of_chunks: usize,
    data_type: DataType,
};

const Table = struct {
    // metadata: []u8,
    columns: []Column,
    num_of_columns: usize,
};

// const ABIDataJustTakeALook = struct {

const SubChunkView = extern struct {
    data: [*]u8,
    offsets: [*]usize,
    lengths: [*]usize,
    num_of_items: usize,
};

const ChunkView = extern struct {
    sub_chunks: [*]SubChunkView,
    referenced_chunks: [*]usize,
    num_of_sub_chunks: usize,
};

const ColumnView = extern struct {
    name: [*]u8,
    name_len: usize,
    chunks: [*]ChunkView,
    num_of_chunks: usize,
    data_type: DataType,
};

const TableView = extern struct {
    columns: [*]ColumnView,
    num_of_columns: usize,
};
