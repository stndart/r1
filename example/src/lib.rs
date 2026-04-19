#[unsafe(no_mangle)]
pub extern "C" fn add(left: i64, right: i64) -> i64 {
    left + right + 6
}
