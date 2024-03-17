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
    cache_actions: Vec<String>,
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
            cache_actions: Vec::new(),
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

    fn record_cache_action(&mut self, action: &str) {
        self.cache_actions.push(action.to_string());
    }

    fn print(&self) {
        println!("hits:{}, misses:{}, evictions:{}", self.hits, self.misses, self.evictions);
    }
}

fn parse_args(args: &[String]) -> (usize, usize, usize, String) {
    let mut s = 0;
    let mut E = 0;
    let mut b = 0;
    let mut t = String::new();

    // loop through and handle parsed options
    let mut opts = getopt::Parser::new(args, "s:E:b:t:");

    while let Some(opt) = opts.next() {

        match opt.unwrap() {
            Opt('s', Some(val)) => s = val.parse().unwrap(),
            Opt('E', Some(val)) => E = val.parse().unwrap(),
            Opt('b', Some(val)) => b = val.parse().unwrap(),
            Opt('t', Some(val)) => t = val,
            _ => {}
        }
    }
   
    (s, E, b, t)
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

            memory_accesses.push(line);
        }
    }

    Ok(memory_accesses)
}

fn parse_memory_access(memory_access: &str, s: usize, b: usize) -> Option<(char, String, usize, usize, usize, usize)> {

     let memory_access_parts: Vec<&str> = memory_access.split_whitespace().collect();
     let operation = memory_access_parts[0].chars().next().unwrap();

     if memory_access_parts.len() >= 2 && memory_access_parts[0] != "I" {
        let address_size_parts: Vec<&str> = memory_access_parts[1].split(',').collect();
        if address_size_parts.len() >= 2 {

        let size = address_size_parts[1].parse::<usize>().unwrap();
        let address = address_size_parts[0].to_string();
        let decimal_address = u64::from_str_radix(address_size_parts[0], 16).unwrap();
        let block_offset = (decimal_address & ((1 << b) - 1)) as usize;
        let set_index = ((decimal_address >> b) & ((1 << s) - 1)) as usize;
        let tag = (decimal_address >> (s + b)) as usize;

        return Some((operation, address, size, tag, set_index, block_offset))
        }
    } 
    
    None
}

fn simulate_cache_accesses(cache: &mut Cache, memory_accesses: &[String], s: usize, b: usize) -> CacheStats{
    let mut cache_stats = CacheStats::new(); 

    for memory_access in memory_accesses {

        if let Some((operation, address, size, tag, set_index, block_offset)) = parse_memory_access(memory_access, s, b) {

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
                cache_stats.increment_hits();
            } else {
                cache_stats.increment_misses();
            }
        }
    }
    cache_stats
}

pub fn main() {
    // collect command line arguments
    let args: Vec<String> = env::args().collect();

    // run parser
    let (s, E, b, t) = parse_args(&args);

    // initialize the cache
    let mut cache = Cache::new(s, E, b); 

    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            let mut cache_stats = simulate_cache_accesses(&mut cache, &memory_accesses, s, b);
            cache_stats.print();
        }
        Err(err) => eprintln!("Error reading trace file: {}", err),
    }
}