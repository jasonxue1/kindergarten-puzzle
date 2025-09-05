pub fn piece_color(i: usize) -> String {
    // Fixed 16-color categorical palette with easily describable hues.
    // Colors are stable and cycle by index%16.
    const PALETTE: [&str; 16] = [
        "red",           // 0
        "orangered",     // 1
        "orange",        // 2
        "gold",          // 3
        "yellowgreen",   // 4
        "green",         // 5
        "mediumseagreen",// 6
        "teal",          // 7
        "deepskyblue",   // 8
        "dodgerblue",    // 9
        "blueviolet",    // 10
        "purple",        // 11
        "fuchsia",       // 12
        "hotpink",       // 13
        "peru",          // 14
        "slategray",     // 15
    ];
    PALETTE[i % PALETTE.len()].to_string()
}
