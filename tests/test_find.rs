use common::util::*;
use std::env;

#[test]
fn test_name_and_prints() {
	
	println!("CWD is {}", env::current_dir().unwrap().display());
    new_ucmd!()
        .args(&["./simple", "-name", "a*c"])
        .succeeds()
        .stdout_only("./simple/abbc");
}

#[test]
fn test_name_and_print_prints() {
	
	println!("CWD is {}", env::current_dir().unwrap().display());
    new_ucmd!()
        .args(&["./simple", "-name", "a*c", "-print"])
        .succeeds()
        .stdout_only("./simple/abbc");
}

#[test]
fn test_name_case_sensitive() {
	
	println!("CWD is {}", env::current_dir().unwrap().display());
    new_ucmd!()
        .args(&["./simple", "-name", "A*C"])
        .succeeds()
        .no_stdout()
        .no_stderr();
}

#[test]
fn test_iname_case_insensitive() {
	
	println!("CWD is {}", env::current_dir().unwrap().display());
    new_ucmd!()
        .args(&["./simple", "-iname", "A*C"])
        .succeeds()
        .stdout_only("./simple/abbc");
}

#[test]
fn test_bad_flag_fails() {
	
	println!("CWD is {}", env::current_dir().unwrap().display());
    new_ucmd!()
        .args(&["./simple", "-bad_flag", "A*C"])
        .fails()
        .stderr_is("Error: Unrecognized flag: '-bad_flag'");
}

#[test]
fn test_bad_directory_detected_but_continues() {
	
	println!("CWD is {}", env::current_dir().unwrap().display());
    new_ucmd!()
        .args(&["./simple", "bad_directory"])
        .succeeds()
        .stderr_is("Error: bad_directory: entity not found")
        .stdout_is("./simple/abbc");}