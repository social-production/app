//! Time limits for the steps of a scenario. Holochain's sweettest has none on zome calls or on
//! shutting a conductor down, so a hang used to run until CI cancelled the whole shard at two
//! hours (`service_requests`, 2026-11-13/14) without saying where it stopped.

use std::future::Future;
use std::time::{Duration, Instant};

/// Longer than any step takes on a slow CI runner (starting a two-agent project network took
/// 4.6 min locally, shutting it down 1.5 min), far shorter than the job's two hours.
const STEP_LIMIT: Duration = Duration::from_mins(15);

/// Run `fut`, failing the test with `label` if it takes longer than `STEP_LIMIT`. Each step's
/// time is printed, so a failing test's captured output shows how far it got. The step is
/// boxed: zome calls and conductor setups are large futures.
pub fn step<'a, T: 'a>(
    label: &'a str,
    fut: impl Future<Output = T> + 'a,
) -> impl Future<Output = T> + 'a {
    let fut = Box::pin(fut);
    async move {
        let start = Instant::now();
        let Ok(out) = tokio::time::timeout(STEP_LIMIT, fut).await else {
            panic!("{label}: still running after {} s", STEP_LIMIT.as_secs());
        };
        eprintln!("  {label}: {:.1} s", start.elapsed().as_secs_f64());
        out
    }
}
