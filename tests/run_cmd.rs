use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::Write;

#[test]
fn shows_error_for_unterminated_parens_function_call() {
    let cases = [
        ("clock(;", "Error at ';': Expect expression."),
        ("print clock(;", "Error at ';': Expect expression."),
    ];

    for (src, expected_err_predicate) in cases {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "{}", src).unwrap();

        cargo_bin_cmd!("codecrafters-interpreter")
            .args(["run"])
            .arg(file.path())
            .assert()
            .failure()
            .append_context("source", src)
            .stderr(predicate::str::contains(expected_err_predicate));
    }
}
