use std::env;
use std::time::Instant;
mod scan;

use sigmars::SigmaCollection;
use crate::scan::process_events_from_dir;

#[tokio::main]
async fn main() {
    let start_time = Instant::now(); // 処理開始時間を記録
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <rules_dir> <log_dir>", args[0]);
        return;
    }

    let rules_dir = &args[1];
    let log_dir = &args[2];
    let rules = SigmaCollection::new_from_dir(rules_dir);
    if let Ok(rules) = rules {
        let log_dir = std::path::PathBuf::from(log_dir);
        process_events_from_dir(&log_dir, rules).await;
    }

    let elapsed_time = start_time.elapsed(); // 経過時間を計算
    println!("Elapsed: {:?}", elapsed_time);
}
