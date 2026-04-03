use crate::types::BenchmarkResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub avg_tokens_per_second: f64,
    pub avg_prompt_eval_time_ms: f64,
    pub avg_eval_time_ms: f64,
    pub total_benchmarks: usize,
    pub best_result: Option<BenchmarkResult>,
    pub worst_result: Option<BenchmarkResult>,
}

pub struct BenchmarkAnalyzer;

impl BenchmarkAnalyzer {
    pub fn analyze(results: &[BenchmarkResult]) -> AnalysisReport {
        if results.is_empty() {
            return AnalysisReport {
                avg_tokens_per_second: 0.0,
                avg_prompt_eval_time_ms: 0.0,
                avg_eval_time_ms: 0.0,
                total_benchmarks: 0,
                best_result: None,
                worst_result: None,
            };
        }

        let total_tokens_per_second: f64 = results.iter().map(|r| r.tokens_per_second).sum();
        let total_prompt_eval: u64 = results.iter().map(|r| r.prompt_eval_time_ms).sum();
        let total_eval: u64 = results.iter().map(|r| r.eval_time_ms).sum();

        let count = results.len() as f64;

        let best = results.iter().max_by(|a, b| {
            a.tokens_per_second
                .partial_cmp(&b.tokens_per_second)
                .unwrap()
        });
        let worst = results.iter().min_by(|a, b| {
            a.tokens_per_second
                .partial_cmp(&b.tokens_per_second)
                .unwrap()
        });

        AnalysisReport {
            avg_tokens_per_second: total_tokens_per_second / count,
            avg_prompt_eval_time_ms: total_prompt_eval as f64 / count,
            avg_eval_time_ms: total_eval as f64 / count,
            total_benchmarks: results.len(),
            best_result: best.cloned(),
            worst_result: worst.cloned(),
        }
    }

    pub fn compare(
        &self,
        baseline: &BenchmarkResult,
        current: &BenchmarkResult,
    ) -> ComparisonResult {
        let tps_diff = current.tokens_per_second - baseline.tokens_per_second;
        let tps_percent = (tps_diff / baseline.tokens_per_second) * 100.0;

        ComparisonResult {
            tokens_per_second_diff: tps_diff,
            tokens_per_second_percent: tps_percent,
            is_improvement: tps_diff > 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub tokens_per_second_diff: f64,
    pub tokens_per_second_percent: f64,
    pub is_improvement: bool,
}
