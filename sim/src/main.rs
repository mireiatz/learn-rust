extern crate getopt;
use getopt::Opt;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};


struct Block {
    tag: u64,
    valid: bool,
}

struct Line {
    blocks: Vec<Block>,
}

struct Set {
    lines: Vec<Line>,
}

struct Cache {
    sets: Vec<Set>,
}

impl Cache {
    fn new(s: usize, e: usize, b: usize) -> Cache {
        let mut sets = Vec::with_capacity(2_usize.pow(s as u32));
        for _ in 0..2_usize.pow(s as u32) {
            let mut lines = Vec::with_capacity(e);
            for _ in 0..e {
                let mut blocks = Vec::with_capacity(1);
                blocks.push(Block {
                    tag: 0,
                    valid: false,
                });
                lines.push(Line { blocks });
            }
            sets.push(Set { lines });
        }
        Cache { sets }
    }
}

fn parse_args(args: &[String]) -> (Cache, usize, usize, usize, String, bool) {
    let mut s = 0;
    let mut E = 0;
    let mut b = 0;
    let mut t = String::new();
    let mut v = false;

    // loop through and handle parsed options
    let mut opts = getopt::Parser::new(args, "hv:s:E:b:t:");

    while let Some(opt) = opts.next() {
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

    // initialize the cache
    let cache = Cache::new(s, E, b); 
    (cache, s, E, b, t, v)
}

fn read_tracefile(filename: &str) -> Result<Vec<String>, std::io::Error> {
    // open and retrieve data from file
    let file_path = format!("../{}", filename);
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    // initialise vector for memory operations
    let mut memory_accesses = Vec::new();

    // loop through file lines
    for line in reader.lines() {
        if let Ok(line) = line {

            // separate parts in each line
            let parts: Vec<&str> = line.split_whitespace().collect();

            //parse parts
            if parts.len() >= 2 {
                // ignore instruction accesses
                if parts[0] != "I" {
                    // store the memory address
                    memory_accesses.push(parts[1].to_string());
                }
            }
        }
    }

    Ok(memory_accesses)
}

pub fn main() {
    // collect command line arguments
    let args: Vec<String> = env::args().collect();

    // run parser
    let (cache, s, E, b, t, v) = parse_args(&args);

    // print parameters
    println!("s: {}", s);
    println!("E: {}", E);
    println!("b: {}", b);
    println!("Tracefile: {}", t);

    if v {
        println!("Verbose mode enabled.");
    }

    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            println!("Memory accesses:");
            for address in &memory_accesses {
                println!("{}", address);
            }
        }
        Err(err) => eprintln!("Error reading trace file: {}", err),
    }
}