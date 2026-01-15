const BlockSize = std.simd.suggestVectorLength(u8).?; // TODO: handle null return (scalar are recommended instead of vectors)
const Block = @Vector(BlockSize, u8);

var global_gpa: std.heap.GeneralPurposeAllocator(.{}) = .init;

pub export fn parse_csv(file_path: [*:0]const u8, delimiter: c_char, quote_char: c_char) callconv(.c) [*c]u8 { //TODO: fix type
    const path: []const u8 = std.mem.span(file_path);

    const file = fs.cwd().openFile(path, .{}) catch |err| {
        c_error.setErrNo(err);
        return @ptrFromInt(0);
    };

    defer file.close();

    const file_size = file.getEndPos() catch |err| {
        c_error.setErrNo(err);
        return @ptrFromInt(0);
    };

    const ptr = std.posix.mmap(
        null,
        file_size,
        std.posix.PROT.READ,
        .{ .TYPE = .SHARED },
        file.handle,
        0,
    ) catch {
        // errno is already set for libc functions
        return @ptrFromInt(0);
    };

    defer std.posix.munmap(ptr);

    var arena: std.heap.ArenaAllocator = .init(global_gpa.allocator());
    const alloc = arena.allocator();
    defer arena.deinit();

    const newlines = prescan_csv(alloc, ptr, '\n') catch |err| {
        c_error.setErrNo(err);
        return @ptrFromInt(0);
    };

    const delimiters = prescan_csv(alloc, ptr, @intCast(delimiter)) catch |err| {
        c_error.setErrNo(err);
        return @ptrFromInt(0);
    };

    const merged = merge_sorted(usize, alloc, newlines, delimiters) catch |err| {
        c_error.setErrNo(err);
        return @ptrFromInt(0);
    };
    std.debug.assert(std.sort.isSorted(usize, merged, {}, std.sort.asc(usize)));

    const quotes = prescan_csv(alloc, ptr, @intCast(quote_char)) catch |err| {
        c_error.setErrNo(err);
        return @ptrFromInt(0);
    };

    std.debug.print("newlines: {any}\n", .{newlines});
    std.debug.print("delimiters: {any}\n", .{delimiters});
    std.debug.print("quote chars: {any}\n", .{quotes});

    // TODO: read in the data (and multi threading)
    return @ptrFromInt(0);
}

fn merge_sorted(comptime T: type, gpa: Allocator, a: []const T, b: []const T) Allocator.Error![]const T {
    comptime {
        switch (@typeInfo(T)) {
            std.builtin.Type.int => {},
            else => @compileError("This function only support interger type, usize, i32, etc..."),
        }
    }

    const merged_array = try gpa.alloc(T, a.len + b.len);
    errdefer {
        gpa.free(merged_array);
    }

    // TODO: make this better
    var ai: usize = 0;
    var bi: usize = 0;
    var i: usize = 0;
    while ((ai < a.len) or (bi > b.len)) {
        if (a[ai] > b[bi]) {
            merged_array[i] = b[bi];
            bi += 1;
        } else {
            merged_array[i] = a[ai];
            ai += 1;
        }

        i += 1;
    }

    if ((i < merged_array.len) and (ai >= a.len)) {
        @memcpy(merged_array[i..], b[bi..]);
    } else if ((i < merged_array.len) and (bi >= b.len)) {
        @memcpy(merged_array[i..], a[ai..]);
    }

    return merged_array;
}

/// Caller owns the returned memory.
fn prescan_csv(gpa: Allocator, ptr: []const u8, char: u8) Allocator.Error![]usize {
    const charVec: Block = @splat(char);
    var positions: ArrayList(usize) = .empty;

    var i: usize = 0;
    while (i + BlockSize < ptr.len) : (i += BlockSize) {
        const block: Block = ptr[i..][0..BlockSize].*;
        const matches = charVec == block;
        var mask: std.bit_set.IntegerBitSet(BlockSize) = .{ .mask = @bitCast(matches) };
        // TODO: maybe this can be optimzed further using @reduce in combination of simd.countTrues, simd.firstTrue, and set firstTrue output to 0, but bitset might still be faster
        while (mask.toggleFirstSet()) |bitpos| {
            try positions.append(gpa, i + bitpos);
        }
    }

    for (ptr[i..], i..) |c, j| {
        if (c == char) {
            try positions.append(gpa, j);
        }
    }

    return positions.toOwnedSlice(gpa);
}

fn parse_file(gpa: Allocator, file: []const u8, delimiters: []const usize, quotes: []const usize) Allocator.Error![]Column {
    var last_idx: usize = 0;
    var delimiter_idx: usize = 0;
    var column_idx: usize = 0;

    const columns = try parse_header(gpa, file, &last_idx, delimiters, &delimiter_idx, quotes);
    errdefer {
        for (columns) |*c| {
            c.free(gpa);
        }
        gpa.free(columns);
    }

    while (last_idx < file.len) {
        const valid_delimiter_idx = get_next_valid_idx(delimiters, delimiter_idx, quotes);
        const idx = blk: {
            if (last_idx > delimiters[valid_delimiter_idx]) {
                break :blk file.len;
            }

            break :blk delimiters[valid_delimiter_idx];
        };

        const data = file[last_idx..idx];

        std.debug.assert(data.len != 0);

        // TODO: handle some rows having more columns than header
        // right now it assumed rectangular csv, completely broken otherwise
        const data_old_idx = columns[column_idx].data.items.len;
        try columns[column_idx].data.appendSlice(gpa, data);
        try columns[column_idx].offsets.append(gpa, data_old_idx);
        try columns[column_idx].data_lens.append(gpa, data.len);

        last_idx = idx + 1;

        delimiter_idx = @min(valid_delimiter_idx + 1, delimiters.len - 1);
        column_idx = (column_idx + 1) % columns.len;
    }

    return columns;
}

// TODO: can we do anything about this long ass function header
/// Parse a file's first line and return array of columns
/// Caller owns the returned memory
fn parse_header(gpa: Allocator, file: []const u8, file_idx: *usize, delimiters: []const usize, delimiter_idx: *usize, quotes: []const usize) Allocator.Error![]Column {
    // TODO: optimization opportunity?, figure out how many column are needed and allocate all at once instead of using arraylist
    var columns: ArrayList(Column) = .empty;
    errdefer {
        for (columns.items) |*c| {
            c.free(gpa);
        }
        columns.deinit(gpa);
    }

    //TODO: optimization opportunity, don't use this function unless newline/delimiters inside quote is enabled
    const first_newline = blk: {
        for (file, 0..) |char, i| {
            if (char == '\n') {
                if (@mod(find_closest_idx(quotes, i), 2) == 1) {
                    break :blk i;
                }
            }
        }

        break :blk file.len;
    };

    while (file_idx.* < first_newline) {
        // this is so ugly lmao
        const valid_delimiter_idx = get_next_valid_idx(delimiters, delimiter_idx.*, quotes);
        const idx = delimiters[valid_delimiter_idx];
        const name = file[file_idx.*..@min(idx, first_newline)];
        const column = try Column.init(gpa, name, .STRING);

        try columns.append(gpa, column);

        file_idx.* += name.len + 1;
        delimiter_idx.* = @min(valid_delimiter_idx + 1, delimiters.len - 1);
    }

    return columns.toOwnedSlice(gpa);
}

/// return the index of `array` where `array[index]` is not within a pair of quotation mark, starting from `index`
fn get_next_valid_idx(array: []const usize, index: usize, quotes: []const usize) usize {
    if (quotes.len == 0) {
        return index;
    }

    var idx = index;
    var valid: bool = false;
    while (!valid) {
        const quote_idx = find_closest_idx(quotes, array[idx]);
        valid = @mod(quote_idx, 2) == 1;
        idx += 1;
    }

    return idx - 1;
}

/// return the index of `array` where `array[index] == value` or `index-1` if `array[index-1] < value < array[index]`
fn find_closest_idx(array: []const usize, value: usize) usize {
    if (array.len == 0) {
        return value;
    }

    var min: usize = 0;
    var max: usize = array.len;
    var current: usize = @divFloor(max, 2);
    while (true) {
        if (array[current] == value) {
            return current;
        } else if (array[current] < value) {
            min = current;
        } else {
            max = current;
        }

        if (max - min == 1) {
            return min;
        }

        current = @divFloor(min + max, 2);
    }
}

const std = @import("std");
const fs = std.fs;
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;
const Csv = @import("root.zig");
const Column = Csv.Column;
const Table = Csv.Table;

const c_error = @import("c_error.zig");

test "prescan_csv" {
    const testing = std.testing;
    const test_alloc = testing.allocator;

    const csv =
        \\Name,Age,Country,Score,Active
        \\John Doe,28,USA,87,Yes
        \\Jane Smith,22,Canada,92,No
        \\Alice Johnson,31,UK,78,Yes
        \\Bob Brown,25,Australia,88,Yes
        \\Charlie Davis,19,USA,95,No
    ;

    const newlines_expect: []const usize = &.{ 29, 52, 79, 106, 136 };
    const delimiters_expect: []const usize = &.{ 4, 8, 16, 22, 38, 41, 45, 48, 63, 66, 73, 76, 93, 96, 99, 102, 116, 119, 129, 132, 150, 153, 157, 160 };

    const newlines_output = try prescan_csv(test_alloc, csv, '\n');
    defer test_alloc.free(newlines_output);

    try testing.expectEqualSlices(usize, newlines_expect, newlines_output);

    const delimiters_output = try prescan_csv(test_alloc, csv, ',');
    defer test_alloc.free(delimiters_output);

    try testing.expectEqualSlices(usize, delimiters_expect, delimiters_output);
}

test "find_closet_index" {
    const testing = std.testing;

    const seed: []const usize = &.{ 0, 1, 4, 7, 8, 12, 43, 126 };
    var actual_idx = find_closest_idx(seed, 0);
    try testing.expectEqual(0, seed[actual_idx]);

    actual_idx = find_closest_idx(seed, 11);
    try testing.expectEqual(8, seed[actual_idx]);

    actual_idx = find_closest_idx(seed, 128);
    try testing.expectEqual(126, seed[actual_idx]);
}

test "merged_ordered" {
    const testing = std.testing;
    const test_alloc = testing.allocator;

    const csv =
        \\Name,Age,Country,Score,Active
        \\John Doe,28,USA,87,Yes
        \\Jane Smith,22,Canada,92,No
        \\Alice Johnson,31,UK,78,Yes
        \\Bob Brown,25,Australia,88,Yes
        \\Charlie Davis,19,USA,95,No
    ;

    const expected: []const usize = &.{ 4, 8, 16, 22, 29, 38, 41, 45, 48, 52, 63, 66, 73, 76, 79, 93, 96, 99, 102, 106, 116, 119, 129, 132, 136, 150, 153, 157, 160 };

    const newlines_output = try prescan_csv(test_alloc, csv, '\n');
    defer test_alloc.free(newlines_output);
    const delimiters_output = try prescan_csv(test_alloc, csv, ',');
    defer test_alloc.free(delimiters_output);

    const merged = try merge_sorted(usize, test_alloc, newlines_output, delimiters_output);
    defer test_alloc.free(merged);

    try testing.expectEqualSlices(usize, expected, merged);
}

test "parse_header" {
    const testing = std.testing;
    const test_alloc = testing.allocator;

    const csv =
        \\Name,Age,Country,Score,Active
        \\John Doe,28,USA,87,Yes
        \\Jane Smith,22,Canada,92,No
        \\Alice Johnson,31,UK,78,Yes
        \\Bob Brown,25,Australia,88,Yes
        \\Charlie Davis,19,USA,95,No
    ;

    const delimiters: []const usize = &.{ 4, 8, 16, 22, 29, 38, 41, 45, 48, 52, 63, 66, 73, 76, 79, 93, 96, 99, 102, 106, 116, 119, 129, 132, 136, 150, 153, 157, 160 };

    var delim_idx: usize = 0;
    var file_idx: usize = 0;
    const columns = try parse_header(test_alloc, csv, &file_idx, delimiters, &delim_idx, &.{});
    defer {
        for (columns) |*c| {
            c.free(test_alloc);
        }
        test_alloc.free(columns);
    }

    // turning comptime string into actual usable string result in really funny looking syntax
    const expected_names: []const []const u8 = &.{ &"Name".*, &"Age".*, &"Country".*, &"Score".*, &"Active".* };

    for (columns, 0..) |c, i| {
        try testing.expectEqualStrings(expected_names[i], c.name);
    }

    try testing.expectEqual(5, delim_idx);
    try testing.expectEqual(30, file_idx);
}

fn compareColumn(expected: Column, actual: Column) error{NotTheSame}!void {
    if (!std.mem.eql(u8, expected.name, actual.name)) {
        std.debug.print("Expected: {s}\nGot: {s}\n", .{ expected.name, actual.name });
        return error.NotTheSame;
    }

    if (!std.mem.eql(u8, expected.data.items, actual.data.items)) {
        std.debug.print("Expected: {s}\nGot: {s}\n", .{ expected.data.items, actual.data.items });
        return error.NotTheSame;
    }

    if (!std.mem.eql(usize, expected.data_lens.items, actual.data_lens.items)) {
        std.debug.print("Expected: {any}\nGot: {any}\n", .{ expected.data_lens.items, actual.data_lens.items });
        return error.NotTheSame;
    }

    if (!std.mem.eql(usize, expected.offsets.items, actual.offsets.items)) {
        std.debug.print("Expected: {any}\nGot: {any}\n", .{ expected.offsets.items, actual.offsets.items });
        return error.NotTheSame;
    }
}

test "parse_file" {
    const testing = std.testing;
    const ta = testing.allocator;

    var arena: std.heap.ArenaAllocator = .init(ta);
    defer arena.deinit();

    const test_alloc = arena.allocator();

    const csv =
        \\Name,Age,Country,Score,Active
        \\John Doe,28,USA,87,Yes
        \\Jane Smith,22,Canada,92,No
        \\Alice Johnson,31,UK,78,Yes
        \\Bob Brown,25,Australia,88,Yes
        \\Charlie Davis,19,USA,95,No
    ;

    const newlines: []const usize = try prescan_csv(test_alloc, csv, '\n');
    const delimiters: []const usize = try prescan_csv(test_alloc, csv, ',');
    const merged: []const usize = try merge_sorted(usize, test_alloc, newlines, delimiters);

    const data = try parse_file(test_alloc, csv, merged, &.{});

    // Column 1: Name
    const expected_data1: ArrayList(u8) =
        .{
            .items = try test_alloc.dupe(u8, &"John DoeJane SmithAlice JohnsonBob BrownCharlie Davis".*),
            .capacity = 53,
        };
    const expected_lens1: ArrayList(usize) =
        .{
            .items = try test_alloc.dupe(usize, &.{ 8, 10, 13, 9, 13 }),
            .capacity = 5,
        };

    const expected_offset1: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 0, 8, 18, 31, 40 }),
        .capacity = 5,
    };

    // Column 2: Age
    const expected_data2: ArrayList(u8) = .{
        .items = try test_alloc.dupe(u8, &"2822312519".*),
        .capacity = 10,
    };
    const expected_lens2: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 2, 2, 2, 2, 2 }),
        .capacity = 5,
    };
    const expected_offset2: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 0, 2, 4, 6, 8 }),
        .capacity = 5,
    };

    // Column 3: Country
    const expected_data3: ArrayList(u8) = .{
        .items = try test_alloc.dupe(u8, &"USACanadaUKAustraliaUSA".*),
        .capacity = 23,
    };
    const expected_lens3: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 3, 6, 2, 9, 3 }),
        .capacity = 5,
    };
    const expected_offset3: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 0, 3, 9, 11, 20 }),
        .capacity = 5,
    };

    // Column 4: Score
    const expected_data4: ArrayList(u8) = .{
        .items = try test_alloc.dupe(u8, &"8792788895".*),
        .capacity = 10,
    };
    const expected_lens4: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 2, 2, 2, 2, 2 }),
        .capacity = 5,
    };
    const expected_offset4: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 0, 2, 4, 6, 8 }),
        .capacity = 5,
    };

    // Column 5: Yes / No
    const expected_data5: ArrayList(u8) = .{
        .items = try test_alloc.dupe(u8, &"YesNoYesYesNo".*),
        .capacity = 13,
    };
    const expected_lens5: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 3, 2, 3, 3, 2 }),
        .capacity = 5,
    };
    const expected_offset5: ArrayList(usize) = .{
        .items = try test_alloc.dupe(usize, &.{ 0, 3, 5, 8, 11 }),
        .capacity = 5,
    };

    const expected_data: []const Column = &.{
        Column{
            .name = "Name",
            .data = expected_data1,
            .offsets = expected_offset1,
            .data_lens = expected_lens1,
            .data_type = .STRING,
        },
        Column{
            .name = "Age",
            .data = expected_data2,
            .offsets = expected_offset2,
            .data_lens = expected_lens2,
            .data_type = .STRING,
        },
        Column{
            .name = "Country",
            .data = expected_data3,
            .offsets = expected_offset3,
            .data_lens = expected_lens3,
            .data_type = .STRING,
        },
        Column{
            .name = "Score",
            .data = expected_data4,
            .offsets = expected_offset4,
            .data_lens = expected_lens4,
            .data_type = .STRING,
        },
        Column{
            .name = "Active",
            .data = expected_data5,
            .offsets = expected_offset5,
            .data_lens = expected_lens5,
            .data_type = .STRING,
        },
    };

    for (expected_data, 0..) |dt, i| {
        try compareColumn(dt, data[i]);
    }
}
