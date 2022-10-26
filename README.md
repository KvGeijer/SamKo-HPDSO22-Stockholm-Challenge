## HPDSO22 Challenge Attempt
This is our attempt of solving the HP Data science open data science challenge, written in the Rust programming language. We mainly tried to make it fast. The code isn't perfect, and the visualization is utter trash.

Event details: https://www.hyperightdataclub.com/hp-data-science-open-stockholm/

Challenge instructions and input files: https://github.com/TheAIFramework/HPDSO22-Stockholm-Challenge

## Run Benchmark
install rust following instructions at: https://www.rust-lang.org/tools/install.

clone git repo using:

```bash
git clone --recurse-submodules git@github.com:ka7801vo/SamKo-HPDSO22-Stockholm-Challenge.git

```
cd into folder, and then run:

```bash
cargo run --release -- <folder 1> <folder 2> ... <folder n>
```
where folder 1..n are paths to directories containing the .bin flight files to parse.
## Unsure About

- How our code actually performs on the badass challenge computer. We are running it on our laptops.

- We used mean edge weight as **dis**similarity during clustering (using kodama rust crate). The problem stated to compute clustering using similarity linkage. Not sure if this wording is significant in some way.

- Our results.

## Possibilities for Improvement
- Instead of acquiring a lock for the graph when adding them all up in the end, we could let each Edge be an Atomic int and let the threads populate the final distance matrix concurrently.

- One could parse airports.csv during compile time and have a pre-generated hash map in memory for direct use, the time saved would however probably be minimal.

- Could have tried to use Divisive Hierarchical Clustering instead, and stop computing divisions when there exist 5 clusters of size 1. (once again, minor improvement since constructing the graph takes so much time). Divisive Hierarchical Clustering can also potentially be parallelized.

- Based on BER structure, we could have just directly read floats at regular intervals and skip the actual parsing.

- Faster hashing algorithm than rusts default secure hash. Tried rustc-hash library, but created too many collisions. Could reversing bit order in the bucketing step improve this? Would this extra step be worth it?

## Participants

* Samuel Selleck
* Kåre von Geijer
* Erik Almbratt

