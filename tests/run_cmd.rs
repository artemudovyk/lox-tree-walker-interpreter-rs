use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::Write;

#[test]
fn outputs_error_to_stderr_for_unterminated_parens_function_call() {
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

#[test]
fn compile_error_on_missing_comma_between_func_parameters() {
    let src = "
        fun foo(a, b c, d, e, f) {}
    ";
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .append_context("source", src)
        .stderr(predicate::str::contains(
            "Missing comma between parameters in function declaration.",
        ));
}
