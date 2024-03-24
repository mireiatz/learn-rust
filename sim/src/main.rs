extern crate getopt;
use getopt::Opt;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

struct Line {
    tag: Option<usize>,
}

struct Set {
    lines: Vec<Line>,
    lru_list: Vec<usize>,
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
                lines.push(Line { tag: None });
            }
            sets.push(Set {
                lines: lines,
                lru_list: vec![0],
            });
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

// Parse command-line arguments and return parameters
fn parse_args(args: &[String]) -> Result<(usize, usize, usize, String), String> {
    let mut s = 0;
    let mut e = 0;
    let mut b = 0;
    let mut t = String::new();

    let mut counts = HashMap::new();
    counts.insert('s', 0);
    counts.insert('E', 0);
    counts.insert('b', 0);
    counts.insert('t', 0);

    let mut opts = getopt::Parser::new(args, "s:E:b:t:"); // Use getopt crate

    while let Some(opt) = opts.next() {
        match opt {
            Ok(Opt(flag, Some(val))) => {
                let count = counts.entry(flag).or_insert(0usize);
                *count += 1;
                if *count > 1 {
                    return Err(format!("duplicate flag -{}", flag));
                }
                match flag {
                    't' => {
                        t = val;
                    }
                    's' | 'E' | 'b' => {
                        let param = val.parse().map_err(|e| format!("invalid value for -{} flag ({})", flag, e))?;
                        if flag == 's' {
                            s = param;
                        } else if flag == 'E' {
                            e = param;
                        } else {
                            b = param;
                        }
                    }
                    _ => return Err(format!("unknown flag: -{}", flag)),
                }
            }
            Ok(Opt(_, None)) => {
                return Err("unexpected option".to_string());
            }
            Err(err) => {
                return Err(format!("{}", err));
            }
        }
    }

    if s == 0 || e == 0 || b == 0 || t.is_empty() {
        return Err("missing required arguments, incorrect command-line format".to_string());
    }

    Ok((s, e, b, t))
}

// Read memory access trace file and return vector of memory accesses
fn read_tracefile(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let file_path = format!("../{}", filename);
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

// Parse memory access string and return set index, tag, and operation
fn parse_memory_access(memory_access: &str, s: usize, b: usize) -> Result<Option<(usize, usize, char)>, String> {
    if memory_access.is_empty() { 
        return Ok(None); 
    }
    let memory_access_parts: Vec<&str> = memory_access.split_whitespace().collect();

    if memory_access_parts.len() >= 2 { 
        if memory_access_parts[0] == "I" { // Skip instruction cache accesses
            return Ok(None);
        }
        let operation = match memory_access_parts[0] {
            "S" | "M" | "L" => memory_access_parts[0].chars().next().unwrap(),
            _ => return Err("invalid operation encountered".to_string())
        };        
        let address_size_parts: Vec<&str> = memory_access_parts[1].split(',').collect();
        if address_size_parts.len() >= 2 {
            let hexadecimal_address = u64::from_str_radix(address_size_parts[0], 16)
                .map_err(|e| format!("failed to parse address ({})", e))?;
            let binary_address = format!("{:0>64b}", hexadecimal_address);
            let set_index_start = 64 - b;
            let tag_start = set_index_start - s;
            let tag = usize::from_str_radix(&binary_address[..tag_start], 2)
                .map_err(|e| format!("failed to parse tag ({})", e))?;
            let set_index = usize::from_str_radix(&binary_address[tag_start..set_index_start], 2)
                .map_err(|e| format!("failed to parse set index ({})", e))?;
            return Ok(Some((set_index, tag, operation)));
        }
    }
    Err("invalid memory access format".to_string())
}

// Update the LRU list based on the accessed line
fn update_lru_list(cache: &mut Cache, set_index: usize, accessed_index: usize) {
    if let Some(position) = cache.sets[set_index]
        .lru_list
        .iter()
        .position(|&i| i == accessed_index)
    {
        cache.sets[set_index].lru_list.remove(position);
    }
    cache.sets[set_index].lru_list.push(accessed_index);
}

// Apply cache simulation logic based on operation and update cache and statistics
fn simulate_cache_access(
    cache: &mut Cache,
    stats: &mut CacheStats,
    operation: char,
    set_index: usize,
    tag: usize,
) -> Result<(), String> {
    match operation {
        'L' | 'S' => {
            let mut found_empty_line = false;

            if set_index >= cache.sets.len() {
                return Err("failed to access cache set".to_string());
            }

            for index in 0..cache.sets[set_index].lines.len() {
                if index >= cache.sets[set_index].lines.len() {
                    return Err("failed to access cache line".to_string());
                }

                if let Some(line_tag) = cache.sets[set_index].lines[index].tag {
                    // If the line is not empty, compare the tags - if they match, it's a hit
                    if line_tag == tag {
                        stats.record_hit();
                        update_lru_list(cache, set_index, index);
                        return Ok(());
                    }
                } else {
                    // If the line is empty, it's a miss and update tag
                    cache.sets[set_index].lines[index].tag = Some(tag); 
                    found_empty_line = true;
                    stats.record_miss();
                    update_lru_list(cache, set_index, index);
                    break;
                }
            }

            // If no hit happened and no empty line was found, evict the LRU line - it's an eviction and update the tag
            if !found_empty_line {
                if let Some(evict_index) = cache.sets[set_index].lru_list.first().cloned() {
                    cache.sets[set_index].lines[evict_index].tag = Some(tag); 
                    stats.record_miss();
                    stats.record_eviction();
                    update_lru_list(cache, set_index, evict_index);
                    return Ok(());
                }
                return Err("eviction failed".to_string());
            }
            return Ok(());
        }
        'M' => {
            // Simulate L operation followed by S operation
            simulate_cache_access(cache, stats, 'L', set_index, tag)?;
            simulate_cache_access(cache, stats, 'S', set_index, tag)?;
            return Ok(());
        }
        _ => {
            return Err(format!("unknown operation: {}", operation));
        }
    }
}

pub fn main() {
    // Collect command line arguments and parse them
    let args: Vec<String> = env::args().collect();
    let (s, e, b, t) = match parse_args(&args) {
        Ok(params) => params,
        Err(err) => {
            eprintln!("Error parsing command-line arguments: {}", err);
            eprintln!("Usage: -- -s <set index bits> -E <lines in set> -b <block bits> -t <tracefile>");
            return;
        }
    };

    // Initialize the cache and stats
    let mut cache = Cache::new(s, e);
    let mut stats = CacheStats::new();

    // Read tracefile and loop through memory accesses
    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            for memory_access in &memory_accesses {

                // Simulate cache behaviour for each memory access
                match parse_memory_access(memory_access, s, b) {
                    Ok(Some((set_index, tag, operation))) => {
                        match simulate_cache_access(&mut cache, &mut stats, operation, set_index, tag) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("Error simulating cache access: {}", err);
                                return;
                            }
                        }
                    }
                    Ok(None) => {
                        continue
                    }
                    Err(err) => {
                        eprintln!("Error parsing memory access: {}", err);
                        return; 
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error reading tracefile {}: {}", t, err);
            return;
        }
    }

    // Print results
    stats.print_stats();
}
