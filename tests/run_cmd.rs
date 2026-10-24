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

#[test]
fn uses_proper_binded_var_for_the_captured_scope() {
    let src = r#"
        var variable = "global";

        {
          fun f() {
            print variable;
          }

          f();
          var variable = "local";
          f();
        }
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("global\nglobal\n"));
}

#[test]
fn mutates_properly_binded_var_for_the_captured_scope() {
    let src = r#"
var count = 0;

{
  // The `counter` function should use the `count`
  // variable from the
  // global scope.
  fun makeCounter() {
    fun counter() {
      // This should increment the `count`
      // variable from the global scope.
      count = count + 1;
      print count;
    }
    return counter;
  }

  var counter1 = makeCounter();
  counter1(); // Should print 1
  counter1(); // Should print 2

  // This variable declaration shouldn't affect
  // our counter.
  var count = 0;

  counter1(); // Should print 3
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1\n2\n3"));
}

#[test]
fn function_name_hoisting() {
    let src = r#"
// This program uses mutual recursion to determine
// if a number is even or odd.
// It also uses a shared threshold variable that
// is used to determine if a number is too large
// to be processed.
{
  var threshold = 50;

  fun isEven(n) {
    if (n == 0) return true;
    if (n > threshold) return false;
    return isOdd(n - 1);
  }

  fun isOdd(n) {
    if (n == 0) return false;
    if (n > threshold) return false;
    return isEven(n - 1);
  }

  print isEven(75);
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn closure_binding_with_functions() {
    let src = r#"
// This function is used in the function `f` below.
fun global() {
  print "global";
}

{
  fun f() {
    global();
  }

  f(); // this should print "global"

  // This function declaration shouldn't affect
  // the usage in `f` above.
  fun global() {
    print "local";
  }

  f(); // this should also print "global"
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("global\nglobal"));
}

#[test]
fn exit_code_65_on_compile_error() {
    let src = r#"
// This program is missing the closing parenthesis
// for the function call
// Hence the compiler error
print clock(;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .code(65);
}

#[test]
fn using_previosly_declared_variable_valie_to_initialize_a_new_variable_should_work() {
    let src = r#"
var quz = "after";
var quz = "before";
// Using a previously declared variable's value to
// initialize a new variable should work
var quz = quz;
print quz;
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("before"));
}

#[test]
fn detects_invalid_local_redeclaratios() {
    let src = r#"
var a = "1";
print a;

var a;
print a;

var a = "2";
print a;

{
  var a = "1";
  var a = "2";
  print a;
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .code(65)
        .stderr(predicate::str::contains(
            "Error at 'a': Already a variable with this name in this scope.",
        ));
}

#[test]
fn detects_invalid_func_parameters_redeclarations() {
    let src = r#"
// Function parameters must have unique names
fun foo(arg, arg) { // expect compile error
    // Function body is irrelevant as the error
    // occurs in parameter list
    "body";
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .code(65)
        .stderr(predicate::str::contains(
            "Error at 'arg': Already a variable with this name in this scope.",
        ));
}

#[test]
fn does_not_trigger_invalid_redeclaration_rules_on_loop_scopes() {
    let src = r#"
// This program uses for loops and block scopes
// to print the updates to the same variable
var baz = "after";

{
    var baz = "before";

    for (var baz = 0; baz < 1; baz = baz + 1) {
         print baz;
         var baz = -1;
         print baz;
    }
}

{
    for (var baz = 0; baz > 0; baz = baz + 1) {}

    var baz = "after";
    print baz;

    for (baz = 0; baz < 1; baz = baz + 1) {
        print baz;
    }
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0\n-1"));
}

#[test]
fn counter_outside_of_for_loop_initializer() {
    let src = r#"
// This program uses a for loop to print the
// numbers from 0 to 2
// The loop initializer is ignored in this loop
var world = 0;
for (; world < 2; world = world + 1) print world;

// This program uses a for loop to print the
// numbers from 0 to 2
// The loop increment clause is ignored in this
// loop
for (var foo = 0; foo < 2;) {
  print foo;
  foo = foo + 1;
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0\n1\n0\n1"));
}

#[test]
fn detects_invalid_return_statements_outside_functions() {
    let src = r#"
fun foo() {
  return "at function scope is ok";
}

return;
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .code(65)
        .stderr(predicate::str::contains(
            "Error at 'return': Can't return from top-level code.",
        ));
}

#[test]
fn does_not_trigger_invalid_return_statement_in_nested_functions() {
    let src = r#"
var count = 0;

{
  // The `counter` function should use the `count`
  // variable from the
  // global scope.
  fun makeCounter() {
    fun counter() {
      // This should increment the `count`
      // variable from the global scope.
      count = count + 1;
      print count;
    }
    return counter;
  }

  var counter1 = makeCounter();
  counter1(); // Should print 1
  counter1(); // Should print 2

  // This variable declaration shouldn't affect
  // our counter.
  var count = 0;

  counter1(); // Should print 3
}
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn print_class_identifier_prints_it() {
    let src = r#"
class Spaceship {}
print Spaceship;
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("Spaceship"));
}

#[test]
fn print_class_instance_prints_class_name_instance() {
    let src = r#"
class Bagel {}
var bagel = Bagel();
print bagel;
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("Bagel instance"));
}

#[test]
fn can_instantiate_multiple_instances_of_the_class() {
    let src = r#"
// Instantiating multiple instances of a class
// should work
class Robot {}
var r1 = Robot();
var r2 = Robot();

print "Created multiple robots:";
print r1;
print r2;
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "Created multiple robots:\nRobot instance\nRobot instance",
        ));
}

#[test]
fn can_init_and_read_class_instance_properties() {
    let src = r#"
// Basic property setting and getting
class Spaceship {}
var falcon = Spaceship();

falcon.name = "Millennium Falcon";
falcon.speed = 75.5;

print "Ship details:";
print falcon.name;
print falcon.speed;
    "#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "Ship details:\nMillennium Falcon\n75.5",
        ));
}

#[test]
fn multiple_object_properties_and_conditional_access() {
    let src = r#"
// Multiple properties and conditional access
class Robot {}
var r2d2 = Robot();

r2d2.model = "Astromech";
r2d2.operational = false;

if (r2d2.operational) {
  print r2d2.model;
  r2d2.mission = "Navigate hyperspace";
  print r2d2.mission;
}
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn multiple_instances_with_properties() {
    let src = r#"
// Multiple instances with properties
class Superhero {}
var batman = Superhero();
var superman = Superhero();

batman.name = "Batman";
batman.called = 18;

superman.name = "Superman";
superman.called = 66;

print "Times " + superman.name + " was called:";
print superman.called;
print "Times " + batman.name + " was called:";
print batman.called;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Times Superman was called:\n66\nTimes Batman was called:\n18",
        ));
}

#[test]
fn functions_should_accept_class_instances_and_mutate_them() {
    let src = r#"
class Wizard {}
var gandalf = Wizard();

gandalf.color = "Grey";
gandalf.power = nil;
print gandalf.color;

// functions should be able to accept class
// instances and get or set properties on them
fun promote(wizard) {
  wizard.color = "White";
  if (false) {
    wizard.power = 100;
  } else {
    wizard.power = 0;
  }
}

promote(gandalf);
print gandalf.color;
print gandalf.power;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Grey\nWhite\n0"));
}

#[test]
fn can_call_class_method() {
    let src = r#"
class Robot {
  beep() {
    print "Beep boop!";
  }
}

var r2d2 = Robot();
r2d2.beep();

Robot().beep();
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Beep boop!\nBeep boop!"));
}

#[test]
fn can_accept_class_instance_as_parameter_to_method() {
    let src = r#"
class Superhero {
  // Methods should be able to accept a parameter
  useSpecialPower(hero) {
    print "Using power: " + hero.specialPower;
  }

  // Methods should be able to accept a parameter
  // of any type
  hasSpecialPower(hero) {
    return hero.specialPower;
  }

  // Methods should be able to accept class
  // instances as parameters and then update their
  // properties
  giveSpecialPower(hero, power) {
    hero.specialPower = power;
  }
}

fun performHeroics(hero, superheroClass) {
  if (superheroClass.hasSpecialPower(hero)) {
    superheroClass.useSpecialPower(hero);
  } else {
    print "No special power available";
  }
}

var superman = Superhero();
var heroClass = Superhero();

if (false) {
  heroClass.giveSpecialPower(superman, "Flight");
} else {
  heroClass.giveSpecialPower(superman, "Strength");
}

performHeroics(superman, heroClass);
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Using power: Strength"));
}
