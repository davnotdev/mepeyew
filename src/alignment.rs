pub unsafe fn pad_raw_slice(
    byte_slice: &[u8],
    min_alignment: usize,
    each_size: usize,
    each_count: usize,
) -> Vec<u8> {
    let padded_each_size = if min_alignment > 0 {
        (each_size + min_alignment - 1) & !(min_alignment - 1)
    } else {
        each_size
    };
    let mut out = vec![0; padded_each_size * each_count];
    for i in 0..each_count {
        for s in 0..each_size {
            out[i * padded_each_size + s] = byte_slice[i * each_size + s];
        }
    }
    out
}
