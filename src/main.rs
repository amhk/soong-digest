mod ansi;
mod error;
mod item;
mod warning;

fn main() {
    let path = match ::std::env::args().nth(1) {
        Some(path) => path,
        None => panic!("missing argument"),
    };
    let contents = std::fs::read_to_string(path).expect("failed to read file");
    let items = error::parse(&contents).expect("failed to parse file");
    for item in items {
        println!("{:?}", item);
    }
}
