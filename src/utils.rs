pub fn format_number_magnitude(number: u64) -> String {
    if number < 1000 {
        number.to_string()
    } else if number < 1000000 {
        format!("{}K", number / 1000)
    } else if number < 1000000000 {
        format!("{}M", number / 1000000)
    } else {
        format!("{}B", number / 1000000000)
    }
}
