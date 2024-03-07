use near_workspaces::result::ExecutionFinalResult;

pub fn transaction_success(result: ExecutionFinalResult) {
    assert!(
        result.is_success(),
        "{}",
        result.into_result().unwrap_err().to_string()
    );
}

pub fn transaction_failure(result: ExecutionFinalResult, expected_error: &str) {
    assert!(result.is_failure());
    assert_eq!(
        result.into_result().unwrap_err().to_string(),
        expected_error
    );
}
