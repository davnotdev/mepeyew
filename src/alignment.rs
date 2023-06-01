pub unsafe fn pad_raw_slice(
    byte_slice: &[u8],
    min_alignment: usize,
    each_size: usize,
    each_count: usize,
) -> Vec<u8> {
    let padded_size = if min_alignment > 0 {
        (each_count * each_size + min_alignment - 1) & !(min_alignment - 1)
    } else {
        each_count * each_size
    };
    let row_size = padded_size / each_count;
    let mut out = vec![0; padded_size];
    for i in 0..each_count {
        for s in 0..each_size {
            out[i * row_size + s] = byte_slice[i * each_size + s];
        }
    }
    out
}
