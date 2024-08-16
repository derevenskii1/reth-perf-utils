//! This module is used to support the display of cached state related metrics.
use super::commons::*;
use revm_utils::{
    metrics::types::{CacheDbRecord, Function},
    time_utils::convert_cycles_to_ns_f64,
};

const COL_WIDTH_BIG: usize = 20;
const COL_WIDTH_MIDDLE: usize = 14;

#[derive(Default, Debug, Copy, Clone)]
struct CacheStat {
    hits: u64,
    misses: u64,
    miss_ratio: f64,
    penalty: f64,
    avg_penalty: f64,
}

const CACHE_STATS_LEN: usize = 5;

#[derive(Debug)]
struct CacheStats {
    functions: [CacheStat; CACHE_STATS_LEN],
}

impl CacheStats {
    fn print_item(&self, function: &str, index: usize) {
        let stats = &self.functions[index];
        println!(
            "{: <COL_WIDTH_BIG$}{:>COL_WIDTH_MIDDLE$}{:>COL_WIDTH_MIDDLE$}{:>COL_WIDTH_BIG$.3}{:>COL_WIDTH_BIG$.3}{:>COL_WIDTH_BIG$.3}",
            function,
            stats.hits,
            stats.misses,
            stats.miss_ratio * 100.0,
            stats.penalty,
            stats.avg_penalty
        );
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        CacheStats {
            functions: [CacheStat::default(); CACHE_STATS_LEN],
        }
    }
}

impl From<&CacheDbRecord> for CacheStats {
    fn from(record: &CacheDbRecord) -> Self {
        let mut cache_stats = CacheStats::default();

        let total_stats = record.access_count();
        let hit_stats = record.hit_stats();
        let miss_stats = record.miss_stats();
        let penalty_stats = record.penalty_stats();

        for index in 0..total_stats.function.len() {
            let total = total_stats.function[index] as f64;
            let misses = miss_stats.function[index] as f64;
            let hits = hit_stats.function[index] as f64;

            cache_stats.functions[index].hits = hit_stats.function[index];
            cache_stats.functions[index].misses = miss_stats.function[index];
            cache_stats.functions[index].miss_ratio = if total > 0.0 {
                misses / total
            } else {
                0.0
            };
            cache_stats.functions[index].penalty = cycles_as_secs(penalty_stats.time.function[index]);
            cache_stats.functions[index].avg_penalty = if misses > 0.0 {
                convert_cycles_to_ns_f64(penalty_stats.time.function[index]) / (1000.0 * misses)
            } else {
                0.0
            };
        }

        let total_hits: u64 = hit_stats.function.iter().sum();
        let total_misses: u64 = miss_stats.function.iter().sum();
        let total_accesses: u64 = total_stats.function.iter().sum();

        cache_stats.functions[CACHE_STATS_LEN - 1].hits = total_hits;
        cache_stats.functions[CACHE_STATS_LEN - 1].misses = total_misses;
        cache_stats.functions[CACHE_STATS_LEN - 1].miss_ratio = if total_accesses > 0 {
            total_misses as f64 / total_accesses as f64
        } else {
            0.0
        };
        cache_stats.functions[CACHE_STATS_LEN - 1].penalty = cycles_as_secs(penalty_stats.time.function.iter().sum());
        cache_stats.functions[CACHE_STATS_LEN - 1].avg_penalty = if total_misses > 0 {
            convert_cycles_to_ns_f64(penalty_stats.time.function.iter().sum())
                / (1000.0 * total_misses as f64)
        } else {
            0.0
        };

        cache_stats
    }
}

impl Print for CacheStats {
    fn print_title(&self) {
        println!("================================================ Metric of State ===========================================");
        println!(
            "{: <COL_WIDTH_BIG$}{:>COL_WIDTH_MIDDLE$}{:>COL_WIDTH_MIDDLE$}{:>COL_WIDTH_BIG$}{:>COL_WIDTH_BIG$}{:>COL_WIDTH_BIG$}",
            "State functions", "Hits", "Misses", "Miss ratio (%)", "Penalty time (s)", "Avg penalty (μs)"
        );
    }

    fn print_content(&self) {
        self.print_item("blockhash", Function::BlockHash as usize);
        self.print_item("code_by_hash", Function::CodeByHash as usize);
        self.print_item("load_account/basic", Function::LoadCacheAccount as usize);
        self.print_item("storage", Function::Storage as usize);
        self.print_item("total", CACHE_STATS_LEN - 1);
    }
}

trait PrintPenalty {
    fn print_penalty(&self);
}

impl PrintPenalty for CacheDbRecord {
    fn print_penalty(&self) {
        println!();
        println!("================Penalty Percentile=============");
        self.penalty_stats().percentile.print_content();
        println!();
    }
}

impl Print for CacheDbRecord {
    fn print(&self, _block_number: u64) {
        Into::<CacheStats>::into(self).print(_block_number);
        self.print_penalty();
    }
}

pub(super) fn print_state_size(block_number: u64, size: usize) {
    println!();
    println!("block_number: {}, State size: {}", block_number, size);
    println!();
}
