use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};
use termimad::*;

struct HumanBytes(usize);

impl Display for HumanBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let size = self.0;
        if size < 1024 {
            write!(f, "{} bytes", size)
        } else if size < 1024 * 1024 {
            write!(f, "{:.2} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

struct HumanBytesPerSec(f64);

impl Display for HumanBytesPerSec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let size = self.0;
        if size < 1024.0 {
            write!(f, "{:.2} b/s", size)
        } else if size < 1024.0 * 1024.0 {
            write!(f, "{:.2} KB/s", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            write!(f, "{:.2} MB/s", size / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.2} GB/s", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

fn test_memory_speed(size: usize, runs: usize, num_cores: usize) -> f64 {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cores)
        .build()
        .unwrap();
    let mut times = Vec::new();

    for _ in 0..runs {
        let src = vec![0u8; size];
        let mut dst = vec![0u8; size];

        let start = Instant::now();
        pool.install(|| {
            dst.par_chunks_mut(1024 * 1024)
                .enumerate()
                .for_each(|(i, chunk)| {
                    let offset = i * 1024 * 1024;
                    chunk.copy_from_slice(&src[offset..offset + chunk.len()]);
                });
        });
        let duration = start.elapsed();
        times.push(duration);
    }

    let total_duration: Duration = times.iter().sum();
    let average_duration = total_duration / runs as u32;
    size as f64 / average_duration.as_secs_f64()
}

fn format_table(block_sizes: &[usize], max_cores: usize, runs: usize) {
    let mut markdown = String::new();
    markdown.push_str("| Block Size |");
    for num_cores in 1..=max_cores {
        markdown.push_str(&format!(" {} Cores |", num_cores));
    }
    markdown.push_str("\n|---|");
    markdown.extend(std::iter::repeat("---|").take(max_cores));

    for &size in block_sizes {
        markdown.push_str(&format!("\n| {} |", HumanBytes(size)));
        for num_cores in 1..=max_cores {
            let speed = test_memory_speed(size, runs, num_cores);
            markdown.push_str(&format!(" {} |", HumanBytesPerSec(speed)));
        }
    }

    let skin = MadSkin::default();
    skin.print_text(&markdown);
}

fn main() {
    let block_sizes = vec![100 * 1024 * 1024, 500 * 1024 * 1024, 1 * 1024 * 1024 * 1024]; // 100MB, 500MB, 1GB
    let runs = 5;
    let max_cores = num_cpus::get();

    format_table(&block_sizes, max_cores, runs);
}
