use std::env;
extern crate getopt;
use getopt::Opt;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Line {
    tag: Option<usize>,
    is_valid: bool,
}

struct Set {
    lines: Vec<Line>,
    access_order: VecDeque<usize>,
}

struct Cache {
    sets: Vec<Set>,
    hits: usize,
    misses: usize,
    evictions: usize,
}

impl Cache {
    // Constructor for Cache struct
    fn new(s: usize, e: usize, b: usize) -> Result<Cache, String> {
        // Calculate total cache size: 2^s * 2^b * E
        match usize::checked_pow(2, s.try_into().unwrap()).and_then(|sets| {
            usize::checked_pow(2, b.try_into().unwrap()).and_then(|blocks| {
                sets.checked_mul(blocks).and_then(|sets_blocks| sets_blocks.checked_mul(e))})
        }) {
            Some(_size) => {
                let mut sets = Vec::with_capacity(2usize.pow(s as u32));
                for _ in 0..2usize.pow(s as u32) {
                    let mut lines = Vec::with_capacity(e);
                    for _ in 0..e {
                        lines.push(Line { 
                            tag: None, 
                            is_valid: false 
                        });
                    }
                    sets.push(Set { 
                        lines: lines, 
                        access_order: VecDeque::new() 
                    });
                }
                Ok(Cache { 
                    sets, 
                    hits: 0, 
                    misses: 0, 
                    evictions: 0 
                })
            }
            None => {
                return Err("cache size exceeds available space (overflow)".to_string());
            }
        }
    }

    // Apply cache simulation logic based on operation and update cache and statistics
    fn simulate_memory_access(&mut self, operation: char, set_index: usize, tag: usize) -> Result<(), String> {
        match operation {
            'L' | 'S' => {
                if set_index >= self.sets.len() {
                    return Err("failed to access cache set".to_string());
                }

                let mut found_empty_line = false;

                for index in 0..self.sets[set_index].lines.len() { 
                    if index >= self.sets[set_index].lines.len() {
                        return Err("failed to access cache line".to_string());
                    }

                    if self.sets[set_index].lines[index].is_valid {
                        // If the line is not empty, compare the tags - if they match, it's a hit
                        if self.sets[set_index].lines[index].tag.unwrap() == tag {
                            self.record_hit();
                            self.update_access_order(set_index, index);
                            return Ok(());
                        }
                    } else {
                        // If the line is empty, the tag has not been found - it's a miss and update the line properties
                        found_empty_line = true;
                        self.sets[set_index].lines[index].tag = Some(tag);
                        self.sets[set_index].lines[index].is_valid = true;
                        self.record_miss();
                        self.update_access_order(set_index, index);
                        break;
                    }
                }

                // If no hit happened and no empty line was found, evict the LRU line - it's an eviction and update the line tag
                if !found_empty_line {
                    if let Some(evict_index) = self.sets[set_index].access_order.pop_back() {
                        self.sets[set_index].lines[evict_index].tag = Some(tag);
                        self.record_miss();
                        self.record_eviction();
                        self.update_access_order(set_index, evict_index);
                        return Ok(());
                    }
                    return Err("eviction failed".to_string());
                }
                return Ok(());
            }
            'M' => {
                // Simulate Load operation followed by Store operation
                self.simulate_memory_access('L', set_index, tag)?;
                self.simulate_memory_access('S', set_index, tag)?;
                return Ok(());
            }
            _ => {
                return Err(format!("unknown operation: {}", operation));
            }
        }
    }

    // Update the LRU order based on the accessed line
    fn update_access_order(&mut self, set_index: usize, accessed_index: usize) {
        let access_order = &mut self.sets[set_index].access_order;

        if let Some(position) = access_order.iter().position(|&i| i == accessed_index) { 
            access_order.remove(position); // Remove accessed_index if it exists
        }
        access_order.push_front(accessed_index); // Add accessed_index at the back
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
        println!("hits:{} misses:{} evictions:{}", self.hits, self.misses, self.evictions);
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

// Read memory access trace file and return memory accesses
fn read_tracefile(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let file_path = format!("../{}", filename);
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

// Parse memory access string and return set index, tag, and operation
fn parse_memory_access(memory_access: &str, s: usize, b: usize) -> Result<Option<(char, usize, usize)>, String> {
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
            _ => return Err("invalid operation encountered".to_string()),
        };
        let address_size_parts: Vec<&str> = memory_access_parts[1].split(',').collect();
        if address_size_parts.len() >= 2 {
            let hexadecimal_address = u64::from_str_radix(address_size_parts[0], 16).map_err(|e| format!("failed to parse address ({})", e))?;
            let binary_address = format!("{:0>64b}", hexadecimal_address);
            let set_index_start = 64 - b;
            let tag_start = set_index_start - s;
            let tag = usize::from_str_radix(&binary_address[..tag_start], 2).map_err(|e| format!("failed to parse tag ({})", e))?;
            let set_index = usize::from_str_radix(&binary_address[tag_start..set_index_start], 2).map_err(|e| format!("failed to parse set index ({})", e))?;
            return Ok(Some((operation, set_index, tag)));
        }
    }
    Err("invalid memory access format".to_string())
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

    // Initialize the cache
    let mut cache = match Cache::new(s, e, b) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error initializing cache: {}", err);
            return;
        }
    };

    // Read tracefile and loop through memory accesses
    match read_tracefile(&t) {
        Ok(memory_accesses) => {
            for memory_access in &memory_accesses {

                // Parse memory accesses
                match parse_memory_access(memory_access, s, b) {
                    Ok(Some((operation, set_index, tag))) => {

                        // Simulate cache behaviour using memory access data
                        match cache.simulate_memory_access(operation, set_index, tag) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("Error simulating cache access: {}", err);
                                return;
                            }
                        }
                    }
                    Ok(None) => continue,
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
    cache.print_stats();
}


#[cfg(test)]
// Tests for parse_args function
#[test]
fn test_parse_args_valid_input() {
    let args = vec![
        "program".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
    ];
    assert_eq!(parse_args(&args), Ok((4, 2, 4, "test_tracefile".to_string())));
}

#[test]
fn test_parse_args_different_order() {
    let args = vec![
        "program".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-b".to_string(),
        "4".to_string(),
    ];
    assert_eq!(parse_args(&args), Ok((4, 2, 4, "test_tracefile".to_string())));
}

#[test]
fn test_parse_args_missing_whitespace() {
    let args = vec![
        "program".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
        "-E2".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-b".to_string(),
        "4".to_string(),
    ];
    assert_eq!(parse_args(&args), Ok((4, 2, 4, "test_tracefile".to_string())));
}

#[test]
fn test_parse_args_missing_arguments() {
    let args = vec![
        "program".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
    ];
    assert!(parse_args(&args).is_err());
}

#[test]
fn test_parse_args_duplicate_flags() {
    let args = vec![
        "program".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-s".to_string(),
        "5".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
    ];
    assert!(parse_args(&args).is_err());
}

#[test]
fn test_parse_args_unknown_flag() {
    let args = vec![
        "program".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-v".to_string(),
        "5".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
    ];
    assert!(parse_args(&args).is_err());
}

#[test]
fn test_parse_args_invalid_values() {
    let invalid_values = vec!["-3", "2.4", "a", "*", "0", ""];
    for invalid_value in invalid_values {
        let args = vec![
            "program".to_string(),
            "-s".to_string(),
            "4".to_string(),
            "-E".to_string(),
            "2".to_string(),
            "-b".to_string(),
            invalid_value.to_string(),
            "-t".to_string(),
            "test_tracefile".to_string(),
        ];
        assert!(parse_args(&args).is_err());
    }
}

#[test]
fn test_parse_args_extra_item() {
    let args = vec![
        "program".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
        "extra".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
    ];
    assert!(parse_args(&args).is_err());
}

#[test]
fn test_parse_args_case_sensitivity_to_upper() {
    let args = vec![
        "program".to_string(),
        "-S".to_string(),
        "4".to_string(),
        "-E".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
    ];
    assert!(parse_args(&args).is_err());
}

#[test]
fn test_parse_args_case_sensitivity_to_lower() {
    let args = vec![
        "program".to_string(),
        "-s".to_string(),
        "4".to_string(),
        "-e".to_string(),
        "2".to_string(),
        "-b".to_string(),
        "4".to_string(),
        "-t".to_string(),
        "test_tracefile".to_string(),
    ];
    assert!(parse_args(&args).is_err());
}

// Tests for read_tracefile function
#[test]
fn test_read_tracefile_ibm() {
    let expected_contents = vec![
       " L 10,4 ", 
       " S 18,4",
       " L 20,4",
       " S 28,4",
       " S 50,4",
    ];
    let result = read_tracefile("traces/ibm.trace");
    assert!(result.is_ok());

    let actual_contents = result.unwrap();
    assert_eq!(actual_contents, expected_contents);
}

#[test]
fn test_read_tracefile_yi() {
    let expected_contents = vec![
        " L 10,1",
        " M 20,1",
        " L 22,1",
        " S 18,1",
        " L 110,1",
        " L 210,1",
        " M 12,1",
    ];

    let result = read_tracefile("traces/yi.trace");
    assert!(result.is_ok());

    let actual_contents = result.unwrap();
    assert_eq!(actual_contents, expected_contents);
}

#[test]
fn test_read_tracefile_yi2() {
    let expected_contents = vec![
        " L 0,1",
        " L 1,1",
        " L 2,1",
        " L 3,1",
        " S 4,1",
        " L 5,1",
        " S 6,1",
        " L 7,1",
        " S 8,1",
        " L 9,1",
        " S a,1",
        " L b,1",
        " S c,1",
        " L d,1",
        " S e,1",
        " M f,1",
    ];

    let result = read_tracefile("traces/yi2.trace");
    assert!(result.is_ok());

    let actual_contents = result.unwrap();
    assert_eq!(actual_contents, expected_contents);
}

#[test]
fn test_read_tracefile_long() {
    assert!(read_tracefile("traces/long.trace").is_ok());
}

#[test]
fn test_read_tracefile_trance() {
    assert!(read_tracefile("traces/trans.trace").is_ok());
}

#[test]
fn test_read_tracefile_test_tracefile() {
    assert!(read_tracefile("test_tracefile").is_err());
}

// Tests for parse_memory_access function
#[test]
fn test_parse_memory_access_valid_input() {
    let memory_access = "S 10,1";
    let s = 4;
    let b = 4;
    assert_eq!(parse_memory_access(memory_access, s, b), Ok(Some(('S', 1, 0))));
}

#[test]
fn test_parse_memory_access_extra_whitespace() {
    let memory_accesses = vec!["S      10,1", "   S 10,1", "S 10,1    "];
    for memory_access in memory_accesses {
        let s = 4;
        let b = 4;
        assert_eq!(parse_memory_access(memory_access, s, b), Ok(Some(('S', 1, 0))));
    }
}

#[test]
fn test_parse_memory_access_instruction_access() {
    let memory_access = "I 10,1";
    let s = 4;
    let b = 4;
    assert_eq!(parse_memory_access(memory_access, s, b), Ok(None));
}

#[test]
fn test_parse_memory_access_invalid_operation() {
    let memory_access = "X 10,1";
    let s = 4;
    let b = 4;
    assert!(parse_memory_access(memory_access, s, b).is_err());
}

#[test]
fn test_parse_memory_access_invalid_format_no_whitespace() {
    let memory_access = "S10,1";
    let s = 4;
    let b = 4;
    assert!(parse_memory_access(memory_access, s, b).is_err());
}

#[test]
fn test_parse_memory_access_invalid_format_no_size() {
    let memory_access = "S 10";
    let s = 4;
    let b = 4;
    assert!(parse_memory_access(memory_access, s, b).is_err());
}

#[test]
fn test_parse_memory_access_invalid_format_no_comma() {
    let memory_access = "S 10:1";
    let s = 4;
    let b = 4;
    assert!(parse_memory_access(memory_access, s, b).is_err());
}

#[test]
fn test_parse_memory_access_invalid_address_value() {
    let memory_access = "S xyz,1";
    let s = 4;
    let b = 4;
    assert!(parse_memory_access(memory_access, s, b).is_err());
}

// Test cache initilisation
#[test]
fn test_cache_new_valid_parameters() {
    let s = 6;
    let e = 2;
    let b = 4;

    match Cache::new(s, e, b) {
        Ok(cache) => {
            assert_eq!(cache.sets.len(), 64); 
            for set in &cache.sets {
                assert_eq!(set.lines.len(), e);

                for line in &set.lines {
                    assert!(!line.is_valid);
                    assert_eq!(line.tag, None); 
                }

                assert_eq!(set.access_order.len(), 0); 
            }
        }
        Err(err) => panic!("Error testing cache: {}", err),
    }
}

#[test]
fn test_cache_new_invalid_size() {
    let s = 1000;
    let e = 16;
    let b = 64;
    assert!(Cache::new(s, e, b).is_err());
}

// Test for simulate_memory_access function
#[test]
fn test_simulate_memory_access_cache_hits() {
    let mut cache = Cache::new(6, 2, 4).unwrap();

    cache.sets[0].lines[0].is_valid = true;
    cache.sets[0].lines[0].tag = Some(100);
    cache.sets[0].access_order.push_back(0);

    assert_eq!(cache.simulate_memory_access('L', 0, 100), Ok(()));
    assert_eq!(cache.hits, 1);
    assert_eq!(cache.misses, 0);
    assert_eq!(cache.evictions, 0);

    assert_eq!(cache.simulate_memory_access('S', 0, 100), Ok(()));
    assert_eq!(cache.hits, 2);
    assert_eq!(cache.misses, 0);
    assert_eq!(cache.evictions, 0);

    assert_eq!(cache.simulate_memory_access('M', 0, 100), Ok(()));
    assert_eq!(cache.hits, 4);
    assert_eq!(cache.misses, 0);
    assert_eq!(cache.evictions, 0);
}

#[test]
fn test_simulate_memory_access_cache_misses() {
    let mut cache = Cache::new(6, 4, 4).unwrap();

    assert_eq!(cache.simulate_memory_access('L', 0, 100), Ok(()));
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.evictions, 0);

    assert_eq!(cache.simulate_memory_access('S', 0, 200), Ok(()));
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 2);
    assert_eq!(cache.evictions, 0);

    assert_eq!(cache.simulate_memory_access('M', 0, 300), Ok(()));
    assert_eq!(cache.hits, 1);
    assert_eq!(cache.misses, 3);
    assert_eq!(cache.evictions, 0);
}

#[test]
fn test_simulate_memory_access_cache_evictions() {
    let mut cache = Cache::new(6, 1, 4).unwrap();

    cache.sets[0].lines[0].is_valid = true;
    cache.sets[0].lines[0].tag = Some(100);
    cache.sets[0].access_order.push_back(0);

    assert_eq!(cache.simulate_memory_access('L', 0, 200), Ok(()));
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.evictions, 1);

    assert_eq!(cache.simulate_memory_access('S', 0, 300), Ok(()));
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 2);
    assert_eq!(cache.evictions, 2);

    assert_eq!(cache.simulate_memory_access('M', 0, 400), Ok(()));
    assert_eq!(cache.hits, 1);
    assert_eq!(cache.misses, 3);
    assert_eq!(cache.evictions, 3);
}

#[test]
fn test_simulate_memory_access_unknown_operation() {
    let mut cache = Cache::new(6, 1, 4).unwrap();

    assert_eq!(cache.simulate_memory_access('X', 0, 100), Err("unknown operation: X".to_string()));
}

// Test for update_access_order function
#[test]
fn test_update_access_order() {
    let mut cache = Cache::new(6, 2, 4).unwrap();

    cache.update_access_order(0, 1);
    assert_eq!(cache.sets[0].access_order, vec![1]);

    cache.update_access_order(0, 2);
    assert_eq!(cache.sets[0].access_order, vec![2, 1]);
 
    cache.update_access_order(0, 1);
    assert_eq!(cache.sets[0].access_order, vec![1, 2]);

    cache.update_access_order(0, 3);
    assert_eq!(cache.sets[0].access_order, vec![3, 1, 2]);
}
