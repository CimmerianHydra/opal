fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/icon.ico"); // Path to your icon file
    res.compile().unwrap();
}