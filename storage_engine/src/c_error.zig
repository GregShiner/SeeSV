const std = @import("std");
const openErr = std.fs.File.OpenError;
const allocErr = std.mem.Allocator.Error;

pub fn setErrNo(err: anyerror) void {
    const errno: c_int = switch (err) {
        openErr.SharingViolation => 13, // EACCES
        openErr.PathAlreadyExists => 17, // EEXIST
        openErr.FileNotFound => 2, // ENOENT
        openErr.AccessDenied => 13, // EACCES
        openErr.PipeBusy => 11, // EAGAIN
        openErr.NoDevice => 6, // ENXIO
        openErr.NameTooLong => 36, // ENAMETOOLONG
        openErr.InvalidUtf8 => 22, // EINVAL
        openErr.InvalidWtf8 => 22, // EINVAL
        openErr.BadPathName => 22, // EINVAL
        openErr.Unexpected => 22, // EINVAL
        openErr.NetworkNotFound => 101, // ENETUNREACH
        openErr.ProcessNotFound => 3, // ESRCH
        openErr.AntivirusInterference => 13, // EACCES
        openErr.PermissionDenied => 13, // EACCES
        openErr.SymLinkLoop => 40, // ELOOP
        openErr.ProcessFdQuotaExceeded => 24, // EMFILE
        openErr.SystemFdQuotaExceeded => 23, // ENFILE
        openErr.SystemResources => 12, // ENOMEM
        openErr.FileTooBig => 27, // EFBIG
        openErr.IsDir => 21, // EISDIR
        openErr.NoSpaceLeft => 28, // ENOSPC
        openErr.NotDir => 20, // ENOTDIR
        openErr.DeviceBusy => 16, // EBUSY
        openErr.FileLocksNotSupported => 95, // ENOTSUP
        openErr.FileBusy => 16, // EBUSY
        openErr.WouldBlock => 11, // EAGAIN
        allocErr.OutOfMemory => 12, // ENOMEM
        else => -1, // Unknown error
    };

    std.c._errno().* = errno;
}
