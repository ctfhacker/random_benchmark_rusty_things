#![feature(drain_filter)]
#![feature(variant_count)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Location {
    address: u64,
    length: u64,
}

impl Location {
    fn new(address: u64, length: u64) -> Location {
        Location { address, length }
    }
}

fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// The maximum free size (as a power of two) to create in the free blocks
const MAX_POW2_FREE_SIZE: u32 = 8;

/// Maximum number of locations to allocate for the free block work set
const FREE_BLOCKS_SIZE: u64 = 1024 * 16;

/// Create a random set of Locations returning the locations and the maximum allocation
fn create_free_blocks() -> (Vec<Location>, u64) {
    let mut result = Vec::new();
    let mut curr_addr = 0;
    let mut max_alloc = 0;
    for _ in 0..(rdtsc() % FREE_BLOCKS_SIZE + 10) {
        // Randomly choose the next size for the allocation
        let next_size = 2_u64.pow(rdtsc() as u32 % MAX_POW2_FREE_SIZE + 1);
        max_alloc = max_alloc.max(next_size);

        // Add this block to the result
        result.push(Location::new(curr_addr, next_size));

        // Bump the address forward
        curr_addr += next_size;

        /*
        // Randomly choose to have an empty gap
        if rdtsc() % 2 == 0 {
            curr_addr += 2_u32.pow(rdtsc() as u32 % 6 + 2);
        }
        */
    }

    (result, max_alloc)
}

fn first_solution(free_blocks: &mut Vec<Location>, alloc: u64) -> Location {
    let next_block = free_blocks
        .iter()
        .filter(|b| b.length >= alloc)
        .min_by_key(|b| b.length)
        .unwrap()
        .clone();

    *free_blocks = free_blocks
        .drain_filter(|b| b.address != next_block.address)
        .collect();

    next_block
}

fn second_solution(free_blocks: &mut Vec<Location>, alloc: u64) -> Location {
    let next_block_idx = free_blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.length >= alloc)
        .min_by_key(|(_, b)| b.length)
        .map(|(index, _)| index);

    free_blocks.swap_remove(next_block_idx.unwrap())
}

fn third_solution(free_blocks: &mut Vec<Location>, alloc: u64) -> Location {
    let next_block_idx = free_blocks
        .iter()
        .enumerate()
        .fold(None, |acc, (idx, val)| match acc {
            None => {
                if val.length >= alloc {
                    Some((idx, val))
                } else {
                    None
                }
            }
            Some((min_idx, min_val)) => {
                if val.length >= alloc && val.length < min_val.length {
                    Some((idx, val))
                } else {
                    Some((min_idx, min_val))
                }
            }
        })
        .expect("Not found")
        .0;

    free_blocks.swap_remove(next_block_idx)
}

fn fourth_solution(free_blocks: &mut Vec<Location>, alloc: u64) -> Location {
    let mut smallest_length = u64::MAX;
    let mut best_index = None;
    for (i, Location { length, .. }) in free_blocks.iter().enumerate() {
        if *length >= alloc && *length < smallest_length {
            smallest_length = *length;
            best_index = Some(i);
        }
    }

    free_blocks.swap_remove(best_index.unwrap())
}

timeloop::impl_enum!(
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum ProfileBlock {
        CreateWork,
        First,
        FilterSwapRemove,
        Fold,
        ForLoop,
    }
);

timeloop::create_profiler!(ProfileBlock);

fn main() {
    const NUM_PROFILE_BLOCKS: u64 = std::mem::variant_count::<ProfileBlock>() as u64;
    const ITERS: usize = 10000;

    println!("Iters: {ITERS}");
    println!("Max size of free blocks: {FREE_BLOCKS_SIZE}");

    timeloop::start_profiler!();

    // Run the benchmark for a number of iterations
    for _ in 0..ITERS {
        // Create the work for each of the test cases
        let (mut work, alloc) = timeloop::time_work!(ProfileBlock::CreateWork, {
            // Create a set of free blocks with the maximum address in this space
            let (free_blocks, max_allocation) = create_free_blocks();
            let alloc: u64 = rdtsc() % max_allocation;

            // Clone the current work for all of the test cases
            let work: Vec<_> = (0..NUM_PROFILE_BLOCKS)
                .map(|_| free_blocks.clone())
                .collect();

            (work, alloc)
        });

        // Reset the answers and finished result arrays
        let mut answers = [Location::default(); NUM_PROFILE_BLOCKS as usize];
        let mut finished = [false; NUM_PROFILE_BLOCKS as usize];

        // Call each test case in a random order
        while !finished.iter().all(|x| *x) {
            let curr_test = (rdtsc() % NUM_PROFILE_BLOCKS) as usize;
            if finished[curr_test] {
                continue;
            }

            let mut curr_work: Vec<Location> = work.pop().unwrap();

            let answer = match curr_test {
                0 => {
                    timeloop::time_work!(ProfileBlock::First, {
                        first_solution(&mut curr_work, alloc)
                    })
                }
                1 => {
                    timeloop::time_work!(ProfileBlock::FilterSwapRemove, {
                        second_solution(&mut curr_work, alloc)
                    })
                }
                2 => {
                    timeloop::time_work!(ProfileBlock::Fold, {
                        third_solution(&mut curr_work, alloc)
                    })
                }
                3 => {
                    timeloop::time_work!(ProfileBlock::ForLoop, {
                        fourth_solution(&mut curr_work, alloc)
                    })
                }
                _ => Location::default(),
            };

            answers[curr_test] = answer;
            finished[curr_test] = true;
        }

        assert!(answers[0] == answers[1]);
        assert!(answers[0] == answers[2]);
        assert!(answers[0] == answers[3]);
    }

    timeloop::print!();
}
