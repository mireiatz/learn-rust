extern crate getopt;
use getopt::Opt;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};


struct Block {
    tag: usize,
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

struct CacheStats {
    hits: usize,
    misses: usize,
    evictions: usize,
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

impl CacheStats {
    fn new() -> Self {
        CacheStats {
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    fn increment_hits(&mut self) {
        self.hits += 1;
    }

    fn increment_misses(&mut self) {
        self.misses += 1;
    }

    fn increment_evictions(&mut self) {
        self.evictions += 1;
    }

    fn print(&self) {
        println!("hits:{}, misses:{}, evictions:{}", self.hits, self.misses, self.evictions);
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
                    let address_parts: Vec<&str> = parts[1].split(',').collect();
                    if address_parts.len() >= 1 {
                        memory_accesses.push(address_parts[0].to_string());
                    }
                }
            }
        }
    }

    Ok(memory_accesses)
}

fn extract_address_parts(address: &str, s: usize, b: usize) -> (usize, usize, usize) {
    let address = u64::from_str_radix(address, 16).unwrap();
    let block_offset = (address & ((1 << b) - 1)) as usize;
    let set_index = ((address >> b) & ((1 << s) - 1)) as usize;
    let tag = (address >> (s + b)) as usize;

    (tag, set_index, block_offset)
}

pub fn main() {
    // collect command line arguments
    let args: Vec<String> = env::args().collect();

    // run parser
    let (mut cache, s, E, b, t, v) = parse_args(&args);

    // print parameters
    println!("s: {}", s);
    println!("E: {}", E);
    println!("b: {}", b);
    println!("Tracefile: {}", t);

    if v {
        println!("Verbose mode enabled.");
    }

    let mut cache_stats = CacheStats::new();

    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            println!("Memory accesses:");
            for address in &memory_accesses {
                let (tag, set_index, block_offset) = extract_address_parts(address, s, b);
                println!("Tag: {}, Set Index: {}, Block Offset: {}", tag, set_index, block_offset);

                let set = &mut cache.sets[set_index];

                let mut found_empty_block = false;
                let mut evicted_block_index = 0;

                for (i, line) in set.lines.iter_mut().enumerate() {
                    let block = &mut line.blocks[0];
                    if !block.valid {
                        // load data into empty block
                        block.tag = tag;
                        block.valid = true;
                        found_empty_block = true;
                        break;
                    } else {
                        // implement eviction policy 
                        evicted_block_index = i;
                    }
                }
            
                if !found_empty_block {
                    // evict the block 
                    let evicted_block = &mut set.lines[evicted_block_index].blocks[0];
                    evicted_block.tag = tag;
                    evicted_block.valid = true;
                    cache_stats.increment_evictions();
                }
            
                if found_empty_block {
                    println!("Cache hit");
                    cache_stats.increment_hits();
                } else {
                    println!("Cache miss");
                    cache_stats.increment_misses();
                }
            }
        }
        Err(err) => eprintln!("Error reading trace file: {}", err),
    }

    cache_stats.print();

}