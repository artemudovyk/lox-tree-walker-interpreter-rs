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
        .stderr(predicate::str::contains(
            "Missing comma between parameters in function declaration.",
        ));
}

#[test]
fn closure() {
    let src = "
        fun makeGreeter() {
          fun greet(name) {
            print \"Hello \" + name;
          }
          return greet;
        }

        var sayHello = makeGreeter();
        sayHello(\"Bob\");
    ";
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello Bob"));
}

#[test]
fn error_when_uncallable_type_called() {
    let src = "85();";
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Can only call functions and classes.",
        ));
}

#[test]
fn error_when_func_called_with_too_many_arguments() {
    let src = "
        fun f(a, b) {
            print a;
            print b;
        }

        f(1, 2, 3, 4);
    ";
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Function \'f\' called with unexpected number of arguments",
        ));
}

#[test]
fn local_function_scope_works_as_expected() {
    let src = "
        var a = 38;

        fun printAndModify() {
            print a;
            var a = 95;
            print a;
        }

        print a;
        a = 68;
        printAndModify();
    ";
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("38\n68\n95\n"));
}

#[test]
fn variable_scoping_works_as_expected() {
    let src = "
        // This program tests variable scoping and
        // shadowing in Lox. It demonstrates:
        // Global variable declarations
        // Function scope access to global variables
        // Block scoping with local variables shadowing
        // outer variables
        // Verification that global variables remain
        // unchanged after shadowing
        var x = 1;
        var y = 2;

        fun printBoth() {
            if (x < y) {
                print \"x is less than y:\";
                print x;
                print y;
            } else {
                print \"x is not less than y:\";
                print x;
                print y;
            }
        }

        {
            var x = 10;
            {
                var y = 20;

                var i = 0;
                while (i < 3) {
                    x = x + 1;
                    y = y - 1;
                    print \"Local x: \";
                    print x;
                    print \"Local y: \";
                    print y;
                    i = i + 1;
                }

                if (x > y) {
                    print \"Local x > y\";
                }

                printBoth();
            }
        }

        if (x == 1 and y == 2) {
            print \"Globals unchanged:\";
            printBoth();
        }
    ";
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Local x: 
11
Local y: 
19
Local x: 
12
Local y: 
18
Local x: 
13
Local y: 
17
x is less than y:
1
2
Globals unchanged:
x is less than y:
1
2",
        ));
}
