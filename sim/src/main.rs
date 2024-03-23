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

fn parse_memory_access(memory_access: &str, s: usize, b: usize) -> Option<(usize, usize)> {
    let memory_access_parts: Vec<&str> = memory_access.split_whitespace().collect();

    if memory_access_parts.len() >= 2 && memory_access_parts[0] != "I" {
        let address_size_parts: Vec<&str> = memory_access_parts[1].split(',').collect();
        if address_size_parts.len() >= 2 {
            let address = u64::from_str_radix(address_size_parts[0], 16).unwrap();
            let binary_address = format!("{:0>64b}", address);
            let set_index_start = 64 - b;
            let tag_index_start = set_index_start-s;
            let tag = usize::from_str_radix(&binary_address[..tag_index_start], 2).unwrap();
            let set_index = usize::from_str_radix(&binary_address[tag_index_start..set_index_start], 2).unwrap();

            return Some((set_index, tag));
        }
    }

    None
}

fn simulate_cache_access(cache: &mut Cache, set_index: usize, tag: usize) -> (bool, bool) {
    println!("Starting simulation for set_index: {}, tag: {}", set_index, tag);

    // Check for hit or empty line
    let mut found_empty_line = false;
    let mut evict_index: Option<usize> = None;

    // Access the lines directly using indexing
    for index in 0..cache.sets[set_index].lines.len() {
        println!("Checking line {}", index);
        let line = &mut cache.sets[set_index].lines[index]; // Borrow the line here
        if let Some(line_tag) = line.tag {
            println!("Line tag found: {}", line_tag);
            if line_tag == tag {
                // Hit, no eviction
                println!("Hit! No eviction needed.");
                line.last_used += 1;

                return (true, false);
            }
        } else {
            // Found an empty line
            println!("Empty line found, setting tag: {}", tag);
            line.tag = Some(tag);
            line.last_used = 0;
            found_empty_line = true;
        }

        // Find eviction candidate
        if evict_index.is_none() || line.last_used > cache.sets[set_index].lines[evict_index.unwrap()].last_used {
            evict_index = Some(index);
        }
    }

    // If no hit and no empty line, evict LRU line
    if !found_empty_line {
        println!("No hit and no empty line.");
        if let Some(evict_index) = evict_index {
            println!("Evicting line at index: {}", evict_index);
            cache.sets[set_index].lines[evict_index].tag = Some(tag);
            cache.sets[set_index].lines[evict_index].last_used = 0;
            println!("Miss with eviction.");

            return (false, true); // Miss, eviction
        }
    }

    println!("Miss without eviction.");
    (false, false) // Miss without eviction
}

pub fn main() {
    // collect command line arguments
    let args: Vec<String> = env::args().collect();

    // run parser
    let (s, E, b, t) = parse_args(&args);

    // initialize the cache
    let mut cache = Cache::new(s, E);

    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            // Initialize counters
            let mut hits = 0;
            let mut misses = 0;
            let mut evictions = 0;
            for address in &memory_accesses {
                // Simulate cache behavior for each memory access
                if let Some((set_index, tag)) = parse_memory_access(address, s, b) {
                    // Simulate cache behavior for each memory access
                    let (hit, eviction) = simulate_cache_access(&mut cache, set_index, tag);
                    if hit {
                        hits += 1;
                    } else {
                        misses += 1;
                        if eviction {
                            evictions += 1;
                        }
                    }
                } else {
                    eprintln!("Error parsing memory access: {}", address);
                }
            }

            // Print results
            println!("hits: {} misses: {} evictions: {}", hits, misses, evictions);
        }
        Err(err) => eprintln!("Error reading trace file: {}", err),
    }
}