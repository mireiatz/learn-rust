extern crate getopt;
use getopt::Opt;
use std::env;

fn parse_args(args: &[String]) -> (usize, usize, usize, String, bool) {
    let mut s = 0;
    let mut E = 0;
    let mut b = 0;
    let mut t = String::new();
    let mut v = false;

    // loop through and handle parsed options
    let mut opts = getopt::Parser::new(args, "hv:s:E:b:t:");
    for opt in &mut opts {
        match opt.unwrap() {
            Opt('h', _) => {
                println!("Usage: path_to_cache_simulator [-hv] -s <num> -E <num> -b <num> -t <file>");
                println!("Options:");
                println!("  -h         Print this help message.");
                println!("  -v         Optional verbose flag.");
                println!("  -s <num>   Number of set index bits.");
                println!("  -E <num>   Number of lines per set.");
                println!("  -b <num>   Number of block offset bits.");
                println!("  -t <file>  Trace file.");
                println!("\nExamples:");
                println!("  linux>  ./sim-ref -s 4 -E 1 -b 4 -t traces/yi.trace");
                println!("  linux>  ./sim-ref -v -s 8 -E 2 -b 4 -t traces/yi.trace");
                std::process::exit(0);
            }
            Opt('v', _) => {
                v = true;
            }
            Opt('s', Some(val)) => s = val.parse().unwrap(),
            Opt('E', Some(val)) => E = val.parse().unwrap(),
            Opt('b', Some(val)) => b = val.parse().unwrap(),
            Opt('t', Some(val)) => t = val,
            _ => {}
        }
    }
    (s, E, b, t, v)
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let (s, E, b, t, v) = parse_args(&args);

    // Print cache parameters
    println!("s: {}", s);
    println!("E: {}", E);
    println!("b: {}", b);
    println!("Tracefile: {}", t);

    if v {
        println!("Verbose mode enabled.");
    }
}