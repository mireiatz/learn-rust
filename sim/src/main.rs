extern crate getopt;
use getopt::Opt;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Line {
    tag: Option<usize>,
    last_used: usize,
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
    fn new(s: usize, e: usize) -> Cache {
        let mut sets = Vec::with_capacity(2usize.pow(s as u32));
        for _ in 0..2usize.pow(s as u32) {
            let mut lines = Vec::with_capacity(e);
            for _ in 0..e {
                lines.push(Line {
                    tag: None,
                    last_used: 0,
                });
            }
            sets.push(Set { lines });
        }
        Cache { sets }
    }
}

impl CacheStats {
    fn new() -> CacheStats {
        CacheStats {
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    fn record_hit(&mut self) {
        self.hits += 1;
    }

    fn record_miss(&mut self) {
        self.misses += 1;
    }

    fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    fn print_stats(&self) {
        println!("hits:{} misses:{} evictions:{}", self.hits, self.misses, self.evictions);
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

fn parse_memory_access(memory_access: &str, s: usize, b: usize) -> Option<(usize, usize, char)> {
    let memory_access_parts: Vec<&str> = memory_access.split_whitespace().collect();
    let operation = memory_access_parts[0].chars().next().unwrap();

    if memory_access_parts.len() >= 2 && memory_access_parts[0] != "I" {
        let address_size_parts: Vec<&str> = memory_access_parts[1].split(',').collect();
        if address_size_parts.len() >= 2 {
            let address = u64::from_str_radix(address_size_parts[0], 16).unwrap();
            let binary_address = format!("{:0>64b}", address);
            let set_index_start = 64 - b;
            let tag_index_start = set_index_start-s;
            let tag = usize::from_str_radix(&binary_address[..tag_index_start], 2).unwrap();
            let set_index = usize::from_str_radix(&binary_address[tag_index_start..set_index_start], 2).unwrap();

            return Some((set_index, tag, operation));
        }
    }

    None
}

fn simulate_cache_access(cache: &mut Cache, stats: &mut CacheStats, operation: char, set_index: usize, tag: usize) {
    match operation {
        'L' | 'S' => {
            println!("Starting simulation for operation: {}, set_index: {}, tag: {}", operation, set_index, tag);

            // Check for hit or empty line
            let mut found_empty_line = false;
            let mut evict_index: Option<usize> = None;

            // Access the lines directly using indexing
            for index in 0..cache.sets[set_index].lines.len() {
                // Check if line is empty or has the same tag
                if let Some(line_tag) = cache.sets[set_index].lines[index].tag {
                    if line_tag == tag {
                        // Hit, no eviction
                        println!("Hit! No eviction needed.");
                        stats.record_hit();
                        cache.sets[set_index].lines[index].last_used += 1;
                        return;
                    }
                } else {
                    // Found an empty line
                    println!("Empty line found, setting tag: {}", tag);
                    cache.sets[set_index].lines[index].tag = Some(tag);
                    cache.sets[set_index].lines[index].last_used = 0;
                    found_empty_line = true;
                }

                // Find eviction candidate
                if evict_index.is_none() || cache.sets[set_index].lines[index].last_used > cache.sets[set_index].lines[evict_index.unwrap()].last_used {
                    evict_index = Some(index);
                }
            }

            // If no hit and no empty line, evict LRU line
            if !found_empty_line {
                if let Some(evict_index) = evict_index {
                    println!("Evicting line at index: {}", evict_index);
                    cache.sets[set_index].lines[evict_index].tag = Some(tag);
                    cache.sets[set_index].lines[evict_index].last_used = 0;
                    println!("Miss with eviction.");
                    stats.record_miss();
                    stats.record_eviction();
                    return;
                }
            }

            println!("Miss without eviction.");
            stats.record_miss();
        }
        'M' => {
             // Load followed by Store (read-modify-write)
             println!("Starting simulation for Modify operation: set_index: {}, tag: {}", set_index, tag);

             // Simulate Load
             simulate_cache_access(cache, stats, 'L', set_index, tag);
 
             // Simulate Store
             simulate_cache_access(cache, stats, 'S', set_index, tag);
        }
        _ => {
            eprintln!("Unknown operation: {}", operation);
        }
    }
}

pub fn main() {
    // collect command line arguments
    let args: Vec<String> = env::args().collect();

    // run parser
    let (s, E, b, t) = parse_args(&args);

    // initialize the cache
    let mut cache = Cache::new(s, E);

    let mut stats= CacheStats::new();

    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            for address in &memory_accesses {
                // Simulate cache behavior for each memory access
                if let Some((set_index, tag, operation)) = parse_memory_access(address, s, b) {
                    // Simulate cache behavior for each memory access
                    simulate_cache_access(&mut cache, &mut stats, operation, set_index, tag);
                }
            }

            // Print results
            stats.print_stats();
        }
        Err(err) => eprintln!("Error reading trace file: {}", err),
    }
}