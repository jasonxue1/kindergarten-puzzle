pub fn piece_color(i: usize) -> String {
    let h = ((i as f64) * 47.0) % 360.0;
    format!("hsl({:.0}, 65%, 75%)", h)
}
