# sim: A CACHE MEMORY SIMULATOR

## Overview

sim is a Rust-based cache memory simulator designed to analyze cache performance based on user-defined parameters and memory access traces. It simulates cache hits, misses, and evictions using an LRU eviction policy.

## Features

* Command-line interface for defining cache parameters

* Set-associative cache structure

* Memory access processing from tracefiles

* LRU (Least Recently Used) eviction policy

* Simulation statistics including hits, misses, and evictions

* Robust testing suite

## Installation

Ensure you have Rust installed. Then, clone the repository and build the program:

`cargo build --release`

## Usage

Execute sim with the following format:

`./sim -s <s> -E <E> -b <b> -t <tracefile>`

where:

* s = Set index bits (determines number of sets: 2^s)

* E = Associativity (number of lines per set)

* b = Block offset bits (block size: 2^b bytes)

* t = Path to tracefile

Example:

`./sim -s 4 -E 2 -b 5 -t traces/example_tracefile.trace`

## Program Execution Flow

1. Parses command-line arguments.

2. Constructs the cache with the specified structure.

3. Reads memory access traces from the tracefile.

4. Simulates memory accesses, tracking hits, misses, and evictions.

5. Outputs statistics in the format:

`hits:X misses:X evictions:X`

## Testing

The testing suite verifies key functionalities, including:

* Command-line argument parsing

* Tracefile handling

* Memory access parsing

* Cache initialisation

* Cache access simulation (hits, misses, evictions)

* LRU access order updating

To run tests:

`cargo test`

## Challenges and Future Development

The primary challenge was implementing the LRU eviction policy efficiently. Future improvements could include:

* Supporting multiple eviction policies (FIFO, Random, etc.)

* Optimising performance for large-scale simulations

