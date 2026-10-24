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

#[test]
fn supports_this_keyword_in_class_methods() {
    let src = r#"
class Cake {
  taste() {
    var adjective = "delicious";
    print "The " + this.flavor + " cake is " + adjective + "!";
  }
}

var cake = Cake();
cake.flavor = "German chocolate";
cake.taste(); // Prints "The German chocolate cake is delicious!"."#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "The German chocolate cake is delicious!",
        ));
}

#[test]
fn this_keyword_in_method_uses_object_it_was_accessed_from_not_invoked_from() {
    let src = r#"
class Egotist {
  speak() {
    print this;
  }
}

var method = Egotist().speak;
method();
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Egotist"));
}

#[test]
fn this_keyword_should_be_bound_to_class_instance_the_method_is_called_on() {
    let src = r#"
class Animal {
  makeSound() {
    print this.sound;
  }

  identify() {
    print this.species;
  }
}

var dog = Animal();
dog.sound = "Woof";
dog.species = "Dog";

var cat = Animal();
cat.sound = "Meow";
cat.species = "Cat";

// The this keyword should be bound to the
// class instance that the method is called on
cat.makeSound = dog.makeSound;
dog.identify = cat.identify;

cat.makeSound(); // expect: Woof
dog.identify(); // expect: Cat
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Woof\nCat"));
}

#[test]
fn does_not_allow_invalid_usage_of_this_keyword() {
    let src = r#"
print this;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Can't use 'this' outside of a class.",
        ));
}

#[test]
fn init_constructor_called_if_present_on_class_instantiation() {
    let src = r#"
class Default {
  init() {
    this.x = "bar";
    this.y = 91;
  }
}
print Default().x;
print Default().y;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("bar\n91"));
}

#[test]
fn constructor_can_be_used_on_instance_itself() {
    let src = r#"
class Counter {
  init(startValue) {
    if (startValue < 0) {
      print "startValue can't be negative";
      this.count = 0;
    } else {
      this.count = startValue;
    }
  }
}

// constructor is called automatically here
var instance = Counter(-51);
print instance.count;

// it should be possible to call the constructor
// on a class instance as well
print instance.init(51).count;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "startValue can't be negative\n0\n51",
        ));
}

#[test]
fn does_not_allow_to_return_value_from_constructor() {
    let src = r#"
class Foo {
  init() {
    return "something else";
  }
}

Foo();
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Can't return a value from an initializer.",
        ));
}

#[test]
fn empty_return_from_class_initializer_allowed() {
    let src = r#"
class Person {
  init() {
    print "world";
    return;
  }
}

Person();
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
fn instance_return_from_class_initializer_not_allowed() {
    let src = r#"
class ThingDefault {
  init() {
    this.x = "foo";
    this.y = 42;
    // constructor should not return the instance
    return this; // expect compile error
  }
}
var out = ThingDefault();
print out;
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
            "Can't return a value from an initializer.",
        ));
}

#[test]
fn inheritance_does_not_change_class_output_ident() {
    let src = r#"
{
  class A {}

  // B is a subclass of A
  class B < A {}

  // C is also a subclass of A
  class C < A {}

  print A();
  print B();
  print C();
}
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A instance\nB instance\nC instance",
        ));
}

#[test]
fn subclass_inherits_superclass_methods() {
    let src = r#"
class Doughnut {
  cook() {
    print "Fry until golden brown.";
  }
}

// BostonCream is a subclass of Doughnut
class BostonCream < Doughnut {}

// BostonCream class should inherit the cook
// method from Doughnut class
BostonCream().cook();
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Fry until golden brown."));
}

#[test]
fn subclass_inherits_superclass_methods_multilevel_inheritance() {
    let src = r#"
class Root {
  getName() {
    print "Root class";
  }
}

class Parent < Root {
  parentMethod() {
    print "Method defined in Parent";
  }
}

class Child < Parent {
  childMethod() {
    print "Method defined in Child";
  }
}

var root = Root();
var parent = Parent();
var child = Child();

// Root methods are available to all
root.getName();
parent.getName();
child.getName();

// Parent methods are available to Parent and Child
parent.parentMethod();
child.parentMethod();

// Child methods are only available to Child
child.childMethod();
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Root class
Root class
Root class
Method defined in Parent
Method defined in Parent
Method defined in Child",
        ));
}

#[test]
fn subclass_runs_superclass_constructors() {
    let src = r#"
class Foo {
  init() {
    this.secret = 42;
  }
}

// Bar is a subclass of Foo
class Bar < Foo {}

// Baz is a subclass of Bar
class Baz < Bar {}

var baz = Baz();

// Baz should inherit the constructor from Foo
// which should set the secret value to 42
print baz.secret;
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn subclass_methods_overrides_same_named_methods_from_superclass() {
    let src = r#"
class A {
  method() {
    print "A method";
  }
}

// B inherits method `method` from A
// and overrides it with a new implementation
class B < A {
  method() {
    print "B method";
  }
}

var b = B();
b.method();  // expect: B method
"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("B method"));
}

#[test]
fn class_methods_are_overriden_top_to_bottom() {
    let src = r#"
class Animal {
  speak() {
    return "Animal speaks";
  }

  makeSound() {
    return "Generic sound";
  }

  communicate() {
    return this.speak() + " : " + this.makeSound();
  }
}

class Dog < Animal {
  speak() {
    return "Dog speaks";
  }

  makeSound() {
    return "Woof";
  }
}

class Puppy < Dog {
  speak() {
    return "Puppy speaks";
  }
}

var animal = Animal();
var dog = Dog();
var puppy = Puppy();

print animal.communicate();
print dog.communicate();
print puppy.communicate();"#;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "{}", src).unwrap();

    cargo_bin_cmd!("codecrafters-interpreter")
        .args(["run"])
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Animal speaks : Generic sound
Dog speaks : Woof
Puppy speaks : Woof",
        ));
}
