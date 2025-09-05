pub fn piece_color(i: usize) -> String {
    // Fixed 8-color palette in the exact order: red, orange, yellow,
    // green, cyan, blue, purple, pink.
    // Colors are stable and cycle by index%8.
    const PALETTE: [&str; 8] = [
        "red",    // 红
        "orange", // 橙
        "yellow", // 黄
        "green",  // 绿
        "cyan",   // 青
        "blue",   // 蓝
        "purple", // 紫
        "pink",   // 粉
    ];
    PALETTE[i % PALETTE.len()].to_string()
}
