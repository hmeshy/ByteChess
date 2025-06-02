// Converts a position to an integer index ("a1" --> 0, "h8" --> 63)
pub fn sq_to_idx(pos: &str) -> usize {
    let mut chars = pos.chars();
    let col = chars.next().unwrap() as usize - 'a' as usize;
    let row = chars.next().unwrap() as usize - '1' as usize;
    (row * 8 + col) as usize
}

// And vice versa
pub fn idx_to_sq(idx: usize) -> String {
    let row = idx / 8;
    let col = idx % 8;
    format!("{}{}", (col as u8 + 'a' as u8) as char, (row as u8 + '1' as u8) as char)
}