pub fn humanize_memory(bytes: usize) -> String {
    let mut bytes = bytes as f64;
    let mut suffix = "B";
    if bytes > 1024.0 {
        bytes /= 1024.0;
        suffix = "KB";
    }
    if bytes > 1024.0 {
        bytes /= 1024.0;
        suffix = "MB";
    }
    if bytes > 1024.0 {
        bytes /= 1024.0;
        suffix = "GB";
    }
    if bytes > 1024.0 {
        bytes /= 1024.0;
        suffix = "TB";
    }
    if bytes > 1024.0 {
        bytes /= 1024.0;
        suffix = "PB";
    }
    format!("{:.2} {}", bytes, suffix)
}
