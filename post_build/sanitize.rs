//! ```cargo
//! [dependencies]
//! dirs = "3.0.1"
//! aho-corasick = "0.7.15"
//! ```

extern crate aho_corasick;
extern crate dirs;

use aho_corasick::AhoCorasickBuilder;
use std::fs;
use std::os::windows::ffi::OsStrExt;

static EXECUTABLE_DIR: &str = concat!(r"target\release\", env!("CARGO_MAKE_PROJECT_NAME"), ".exe");

// Remapping the prefix is unstable and broken so we'll have to do this
fn main() {
  let directory = dirs::home_dir().expect("home directory").into_os_string();
  // Windows allows you to have unicode names
  let wide_bytes = directory.encode_wide().collect::<Vec<u16>>();
  let pattern = String::from_utf16(&wide_bytes).unwrap();

  let haystack = fs::read(EXECUTABLE_DIR).expect("binary bytes");

  let searcher = AhoCorasickBuilder::new().build(&[pattern]);
  let mut res = Vec::with_capacity(haystack.len());

  searcher.replace_all_with_bytes(&haystack, &mut res, |_, recv_bytes, dst| {
    dst.extend_from_slice(&vec![0; recv_bytes.len()]);
    true
  });

  assert_eq!(haystack.len(), res.len());

  let _ = fs::write(EXECUTABLE_DIR, res);
}
