#[link(name = "dll_outer.dll", kind = "dylib")]
unsafe extern "C" {
    fn add(left: i64, right: i64) -> i64;
}

fn main() {
    let x = unsafe { add(1, 2) };
    println!("Hello, world! {x}");
}
