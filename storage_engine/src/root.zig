//! By convention, root.zig is the root source file when making a library.
const DataType = enum {
    INT,
    FLOAT,
    STRING,
};

// const ZigDataDoNotTouch = struct {

pub const Chunk = struct {

    // pub fn put(i: usize) []u8 {
    //     return data
    // }
};

pub const Column = struct {
    name: []const u8,
    data: ArrayList(u8),
    offsets: ArrayList(usize),
    data_lens: ArrayList(usize),
    data_type: DataType,

    /// duplicate `name` into heap memory
    pub fn init(gpa: Allocator, name: []const u8, data_type: DataType) Allocator.Error!Column {
        const column_name = try gpa.dupe(u8, name);

        return Column{
            .name = column_name,
            .data = .empty,
            .offsets = .empty,
            .data_lens = .empty,
            .data_type = data_type,
        };
    }

    pub fn free(self: *Column, gpa: Allocator) void {
        gpa.free(self.name);
        self.data.deinit(gpa);
        self.offsets.deinit(gpa);
        self.data_lens.deinit(gpa);
    }
};

pub const Table = struct {
    // metadata: []u8,
    columns: []Column,
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

const std = @import("std");
const ArrayList = std.ArrayList;
const Allocator = std.mem.Allocator;
