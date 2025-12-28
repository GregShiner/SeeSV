const BlockSize = std.simd.suggestVectorLength(u8).?; // TODO: handle null return (scalar are recommended instead of vectors)
const Block = @Vector(BlockSize, u8);

var gpa: std.heap.GeneralPurposeAllocator(.{}) = .init;

pub export fn parse_csv(file_path: [*:0]const u8, delimiter: c_char) callconv(.c) *c_int { //TODO: fix type
    const path: []const u8 = std.mem.span(file_path);

    const file = fs.cwd().openFile(path, .{}) catch |err| {
        c_error.setErrNo(err);
        return undefined;
    };

    defer file.close();

    const file_size = file.getEndPos() catch |err| {
        c_error.setErrNo(err);
        return undefined;
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
        return undefined;
    };

    defer std.posix.munmap(ptr);

    var arena: std.heap.ArenaAllocator = .init(gpa.allocator());
    const alloc = arena.allocator();
    defer arena.deinit();

    const newlines = prescan_csv(alloc, ptr, '\n') catch |err| {
        c_error.setErrNo(err);
        return undefined;
    };

    const delimiters = prescan_csv(alloc, ptr, @intCast(delimiter)) catch |err| {
        c_error.setErrNo(err);
        return undefined;
    };

    std.debug.print("newlines: {any}\n", .{newlines});
    std.debug.print("delimiters: {any}\n", .{delimiters});

    // TODO: read in the data (and multi threading)
    return undefined;
}

/// Caller owns the returned memory.
fn prescan_csv(alloc: Allocator, ptr: []const u8, char: u8) error{OutOfMemory}![]usize {
    const charVec: Block = @splat(char);
    var positions: ArrayList(usize) = .empty;

    var i: usize = 0;
    while (i + BlockSize < ptr.len) : (i += BlockSize) {
        const block: Block = ptr[i..][0..BlockSize].*;
        const matches = charVec == block;
        var mask: std.bit_set.IntegerBitSet(BlockSize) = .{ .mask = @bitCast(matches) };
        // TODO: maybe this can be optimzed further using @reduce in combination of simd.countTrues, simd.firstTrue, and set firstTrue output to 0, but bitset might still be faster
        while (mask.toggleFirstSet()) |bitpos| {
            try positions.append(alloc, i + bitpos);
        }
    }

    for (ptr[i..], i..) |c, j| {
        if (c == char) {
            try positions.append(alloc, j);
        }
    }

    return positions.toOwnedSlice(alloc);
}

const std = @import("std");
const fs = std.fs;
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;

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
