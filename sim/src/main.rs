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
    // Constructor for Cache struct
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
    // Constructor for CacheStats struct
    fn new() -> CacheStats {
        CacheStats {
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    // Increase cache hits count
    fn record_hit(&mut self) {
        self.hits += 1;
    }

    // Increase cache misses count
    fn record_miss(&mut self) {
        self.misses += 1;
    }

    // Increase cache evictions count
    fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    // Print cache statistics
    fn print_stats(&self) {
        println!(
            "hits:{} misses:{} evictions:{}",
            self.hits, self.misses, self.evictions
        );
    }
}

// Parse command line arguments and return parameters
fn parse_args(args: &[String]) -> (usize, usize, usize, String) {
    let mut s = 0;
    let mut e = 0;
    let mut b = 0;
    let mut t = String::new();

    // Use getopt crate
    let mut opts = getopt::Parser::new(args, "s:E:b:t:");

    while let Some(opt) = opts.next() {
        match opt.unwrap() {
            Opt('s', Some(val)) => s = val.parse().unwrap(),
            Opt('E', Some(val)) => e = val.parse().unwrap(),
            Opt('b', Some(val)) => b = val.parse().unwrap(),
            Opt('t', Some(val)) => t = val,
            _ => {}
        }
    }

    (s, e, b, t)
}

// Read memory access trace file and return vector of memory accesses
fn read_tracefile(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let file_path = format!("../{}", filename);
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

// Parse memory access string and return set index, tag, and operation
fn parse_memory_access(memory_access: &str, s: usize, b: usize) -> Option<(usize, usize, char)> {
    let memory_access_parts: Vec<&str> = memory_access.split_whitespace().collect();

    if memory_access_parts.len() >= 2 && memory_access_parts[0] != "I" {
        let operation = memory_access_parts[0].chars().next().unwrap();
        let address_size_parts: Vec<&str> = memory_access_parts[1].split(',').collect();
        if address_size_parts.len() >= 2 {
            let address = u64::from_str_radix(address_size_parts[0], 16).unwrap(); // Parse hexadecimal address
            let binary_address = format!("{:0>64b}", address);
            let set_index_start = 64 - b;
            let tag_index_start = set_index_start - s;
            let tag = usize::from_str_radix(&binary_address[..tag_index_start], 2).unwrap();
            let set_index =
                usize::from_str_radix(&binary_address[tag_index_start..set_index_start], 2)
                    .unwrap();
            return Some((set_index, tag, operation));
        }
    }
    None
}

// Apply cache simulation logic based on operation and update cache and statistics
fn simulate_cache_access(
    cache: &mut Cache,
    stats: &mut CacheStats,
    operation: char,
    set_index: usize,
    tag: usize,
) {
    match operation {
        'L' | 'S' => {
            let mut found_empty_line = false;
            let mut evict_index: Option<usize> = None;

            for index in 0..cache.sets[set_index].lines.len() {
                if let Some(line_tag) = cache.sets[set_index].lines[index].tag {
                    // If the line is not empty, compare the tags - if they match, it's a hit
                    if line_tag == tag {
                        cache.sets[set_index].lines[index].last_used += 1; // Update last used value for LRU eviction policy purposes
                        stats.record_hit();
                        return;
                    }
                } else {
                    // If the line is empty, update its values
                    cache.sets[set_index].lines[index].tag = Some(tag); // Set the tag
                    cache.sets[set_index].lines[index].last_used = 0; // Update last used value for LRU eviction policy purposes
                    found_empty_line = true;
                }

                // Find eviction candidate
                if evict_index.is_none()
                    || cache.sets[set_index].lines[index].last_used
                        > cache.sets[set_index].lines[evict_index.unwrap()].last_used
                {
                    evict_index = Some(index);
                }
            }

            // If no hit happened and no empty line was found, evict the LRU line
            if !found_empty_line {
                if let Some(evict_index) = evict_index {
                    cache.sets[set_index].lines[evict_index].tag = Some(tag); // Set the tag
                    cache.sets[set_index].lines[evict_index].last_used = 0; // Update last used value for LRU eviction policy purposes
                    stats.record_miss();
                    stats.record_eviction();
                    return;
                }
            }

            // If no hit or eviction happened, then it's a miss
            stats.record_miss();
        }
        'M' => {
            // Simulate L operation followed by S operation
            simulate_cache_access(cache, stats, 'L', set_index, tag);
            simulate_cache_access(cache, stats, 'S', set_index, tag);
        }
        _ => {
            eprintln!("Unknown operation: {}", operation);
        }
    }
}

pub fn main() {
    // Collect command line arguments and parse them
    let args: Vec<String> = env::args().collect();
    let (s, e, b, t) = parse_args(&args);

    // Initialize the cache and stats
    let mut cache = Cache::new(s, e);
    let mut stats = CacheStats::new();

    // Read tracefile and loop through memory accesses
    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            for address in &memory_accesses {
                // Simulate cache behaviour for each memory access
                if let Some((set_index, tag, operation)) = parse_memory_access(address, s, b) {
                    simulate_cache_access(&mut cache, &mut stats, operation, set_index, tag);
                }
            }

            // Print results
            stats.print_stats();
        }
        Err(err) => eprintln!("Error reading trace file: {}", err),
    }
}
