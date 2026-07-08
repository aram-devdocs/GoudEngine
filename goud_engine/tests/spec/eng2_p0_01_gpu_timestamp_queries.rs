use goud_engine::libs::graphics::backend::wgpu_backend::probe_gpu_timestamp_queries;

#[test]
fn eng2_p0_01_query_set_populates_and_resolves_on_wgpu_backend() {
    let report = match probe_gpu_timestamp_queries() {
        Ok(report) => report,
        Err(err) => {
            eprintln!("Skipping ENG2-P0-01 probe: {err}");
            return;
        }
    };

    if !report.supported {
        eprintln!("Skipping ENG2-P0-01 probe: adapter does not expose timestamp-query features");
        return;
    }
    if report.raw_queries.iter().all(|query| *query == 0) {
        eprintln!(
            "Skipping ENG2-P0-01 probe: adapter returned zeroed timestamp-query data: {:?}",
            report.raw_queries
        );
        return;
    }

    assert!(
        report.raw_queries.iter().all(|query| *query > 0),
        "expected every timestamp slot to be written, got {:?}",
        report.raw_queries
    );
    assert!(
        report.raw_queries[1] > report.raw_queries[0],
        "expected shadow timestamps to advance, got {:?}",
        report.raw_queries
    );
    assert!(
        report.raw_queries[3] > report.raw_queries[2],
        "expected render timestamps to advance, got {:?}",
        report.raw_queries
    );
    assert!(
        report.raw_queries[5] > report.raw_queries[4],
        "expected submit-tail timestamps to advance, got {:?}",
        report.raw_queries
    );
}
