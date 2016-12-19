extern crate glob;

use glob::Pattern;
use glob::PatternError;
use std::error::Error;
use std::path::Path;
use std::fs::{self, DirEntry};
use std::io::stderr;
use std::io::Write;

/// A basic interface that can be used to determine whether a directory entry
/// is what's being searched for. To a first order approximation, find consists
/// of building a chain of Matcher objets, and then w,alking a directory tree,
/// passing each entry to the chaing of Matchers.
trait Matcher {
  /// Returns whether the given file matches the object's predicate.
  fn matches(&self, file_info: &DirEntry) -> bool;

  /// Returns whether the matcher has any side-effects. Iff no such matcher
  /// exists in the chain, then the filename will be printed to stdout. While
  /// this is a compile-time fact for most matchers, it's run-time for matchers
  /// that contain a collection of sub-Matchers.
  fn has_side_effects(&self) -> bool;

}


/// This matcher just prints the name of the file to stdout.
struct Printer {}

impl Matcher for Printer {
 fn matches(&self, file_info: &DirEntry) -> bool {
   if let Some(x) = file_info.path().to_str() {
    println!("{}", x);
   }
   true
  }

  fn has_side_effects(&self) -> bool {
    true
  }
}

/// This matcher makes a case-sensitive comparison of the name against a 
/// shell wildcard pattern. See glob::Pattern for details on the exact
/// syntax.
pub struct NameMatcher {
  pattern: Pattern,

}

impl NameMatcher {
  fn new(pattern_string : &String) -> Result<NameMatcher, PatternError> {
    let p = try!(Pattern::new(pattern_string));
    Ok(NameMatcher{ pattern : p})
  }
}

impl Matcher for NameMatcher {
  fn matches(&self, file_info: &DirEntry) -> bool {
    if let Ok(x) = file_info.file_name().into_string() {
    return self.pattern.matches(x.as_ref())
    }
    false
  }

  fn has_side_effects(&self) -> bool {
    false
  }
}

/// This matcher makes a case-insensitive comparison of the name against a 
/// shell wildcard pattern. See glob::Pattern for details on the exact
/// syntax.
struct CaselessNameMatcher {
  pattern: Pattern,

}

impl CaselessNameMatcher {
  fn new(pattern_string : &String) -> Result<NameMatcher, PatternError> {
    let p = try!(Pattern::new(&pattern_string.to_lowercase()));
    Ok(NameMatcher{ pattern : p})
  }
}

impl Matcher for CaselessNameMatcher {
  fn matches(&self, file_info: &DirEntry) -> bool {
    if let Ok(x) = file_info.file_name().into_string() {
      return self.pattern.matches(x.to_lowercase().as_ref())
    }
    false
  }

  fn has_side_effects(&self) -> bool {
    false
  }
}

/// This matcher contains a collection of other matchers. A file only matches
/// if it matches ALL the contained sub-matchers. For sub-matchers that have 
/// side effects, the side effects occur in the same order as the sub-matchers
/// were pushed into the collection.
struct AndMatcher<> {
  submatchers : Vec<Box<Matcher>>,
}

impl AndMatcher {
  fn push(&mut self, matcher : Box<Matcher>) {
    self.submatchers.push(matcher);
  }

  fn new() -> AndMatcher {
    AndMatcher{
      submatchers : Vec::new()
    }
  }
}


impl Matcher for AndMatcher {
  fn matches(&self, file_info: &DirEntry) -> bool {
    for matcher in &self.submatchers{
      if !matcher.matches(file_info) {
        return false;
      }
    }
    true
  }

  fn has_side_effects(&self) -> bool {
    for matcher in &self.submatchers{
      if matcher.has_side_effects() {
        return true;
      }
    }
    false
  }
}


/// Builds a single AndMatcher containing the Matcher objects corresponding
/// to the passed in predicate arguments.
fn build_top_level_matcher(args : &[String]) -> Result<Box<Matcher>, Box<std::error::Error>> {
  let mut top_level_matcher = AndMatcher::new();

  // can't use getopts for a variety or reasons:
  // order ot arguments is important
  // arguments can start with + as well as -
  // multiple-character flags don't start with a double dash
  let mut i = 0;
  while i < args.len() {
    let submatcher = match args[i].as_ref() {
      "-print" => Box::new(Printer{}) as Box<Matcher>,
      "-name" => {
          i += 1;
          if i >= args.len() {
            return Err(From::from("Must supply a pattern with -name"));
          }
          Box::new(try!(NameMatcher::new(&args[i])))
        },
      "-iname" => {
          i += 1;
          if i >= args.len() {
            return Err(From::from("Must supply a pattern with -iname"));
          }
          Box::new(try!(CaselessNameMatcher::new(&args[i])))
        },
      _ => return Err(From::from(format!("Unrecognized flag: '{}'", args[i])))
    };
    top_level_matcher.push(submatcher);
    i += 1;
  }

  if !top_level_matcher.has_side_effects() {
    top_level_matcher.push(Box::new(Printer{}));
  }
  Ok(Box::new(top_level_matcher))
}

struct PathsAndMatcher {
  matcher : Box<Matcher>,
  paths : Vec<String>,
}

fn parse_args(args : &[String]) -> Result<PathsAndMatcher, Box<Error>> {
  let mut paths : Vec<String> = Vec::new();
  let mut i = 0;

  while i < args.len() && !args[i].starts_with('-') {
    paths.push(args[i].clone());
    i += 1;
  }
  if i == 0 {
    paths.push(".".to_string());
  }
  let matcher = try!(build_top_level_matcher(&args[i ..]));
  Ok(PathsAndMatcher{ matcher : matcher, paths : paths})
}

fn process_dir(dir : &Path, matcher : &Box<Matcher>) 
  -> Result<i32, Box<Error>> {
  let mut found_count = 0;
  match fs::read_dir(dir) {
    Ok(entry_results) => {
      for entry_result in entry_results {
        let entry = try!(entry_result);
            let path : std::path::PathBuf = entry.path();
            if matcher.matches(&entry) {
              found_count += 1;
            }
            if path.is_dir() {
                try!(process_dir(&path, matcher));
            }
      }
    },
    Err(e) => {
      writeln!(&mut stderr(), 
        "Error: {}: {}", 
        dir.to_string_lossy(), e.description()).unwrap();
    }
  }
  Ok(found_count)
}


fn do_find(args : &[String]) -> Result<i32, Box<Error>> {

  let paths_and_matcher = try!(parse_args(args));
  let mut found_count = 0;
  for path in paths_and_matcher.paths {
    let dir = Path::new(&path);
    found_count += try!(process_dir(&dir, &paths_and_matcher.matcher));
  }
  Ok(found_count)
}

fn print_help() {
  println!("Usage: find [path...] [expression]

If no path is supplied then the current working directory is used by default.

Early alpha implementation. Currently the only expressions supported are
 -print
 -name case-sensitive_filename_pattern
 -iname case-insensitive_filename_pattern
");
}



pub fn uumain(args: Vec<String>) -> i32 {

  for arg in &args {
    match arg.as_ref() {
      "-help" | "--help" | "-?" | "-h" => {
        print_help();
        return 0;
      }
      _=> ()
    }
  }
  match do_find(&args[1..]) {
    Ok(_) => 0,
    Err(e) => {
      writeln!(&mut stderr(), "Error: {}", e).unwrap();
      1},
  }
}
