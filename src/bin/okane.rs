use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };
    let mut ser = String::new();
    match file.read_to_string(&mut ser) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => (),
    }
    let res = match okane::converter::iso_camt053::print_camt(ser) {
        Err(why) => panic!("couldn't parse {}: {}", path.display(), why),
        Ok(res) => res,
    };
    println!("{}", res);
}
