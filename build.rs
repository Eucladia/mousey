use build::*;

fn main() {
  generate();
}

#[cfg(windows)]
mod build {
  use std::env;
  use std::fmt::Write;
  use std::fs;
  use std::path::Path;

  static PATH: &str = r#"resources\mousey.rc"#;

  pub fn generate() {
    let mut rc = String::new();
    let (file_name, author, year) = parse_cargo();

    let _ = writeln!(rc, "#pragma code_page(65001)");
    let _ = writeln!(rc, "\n1 VERSIONINFO");

    write_version(&mut rc);
    rc.push_str("\n\n{\n");

    write_string_file_info(&mut rc, &file_name, &author, year);
    write_var_info(&mut rc);

    rc.push_str("\n}\n");

    fs::write(PATH, rc).unwrap();

    embed_resource::compile(PATH);
  }

  fn write_version(string: &mut String) {
    const VERSION: &str = concat!(
      env!("CARGO_PKG_VERSION_MAJOR"),
      ",",
      env!("CARGO_PKG_VERSION_MINOR"),
      ",",
      env!("CARGO_PKG_VERSION_PATCH"),
      ",",
      "0"
    );

    let defaults =
      "FILESUBTYPE 0x0\nFILEFLAGS 0x0\nFILETYPE 0x1\nFILEFLAGSMASK 0x3f\nFILEOS 0x40004";

    let _ = write!(
      string,
      "\nPRODUCTVERSION {version}\nFILEVERSION {version}\n{defaults}",
      version = VERSION,
      defaults = defaults
    );
  }

  fn write_string_file_info(string: &mut String, file_name: &str, author: &str, year: u32) {
    const VERSION: &str = concat!(
      env!("CARGO_PKG_VERSION_MAJOR"),
      ".",
      env!("CARGO_PKG_VERSION_MINOR"),
      ".",
      env!("CARGO_PKG_VERSION_PATCH")
    );

    let pad = " ".repeat(2);

    let _ = write!(string, "{}", &pad);
    let _ = writeln!(string, r#"BLOCK "StringFileInfo" {{"#);
    let _ = write!(string, "{}", &pad);

    // Language = 0
    let _ = writeln!(string, r#"BLOCK "{:04x}04b0" {{"#, 0);
    // Version
    let _ = writeln!(
      string,
      r#"{}VALUE "ProductVersion", "{}""#,
      pad.repeat(3),
      VERSION
    );
    let _ = writeln!(
      string,
      r#"{}VALUE "FileVersion", "{}""#,
      pad.repeat(3),
      VERSION
    );
    // Other Attributes
    let _ = writeln!(
      string,
      r#"{}VALUE "ProductName", "{}""#,
      pad.repeat(3),
      file_name
    );
    let _ = writeln!(
      string,
      r#"{}VALUE "InternalName", "{}""#,
      pad.repeat(3),
      file_name
    );
    let _ = writeln!(
      string,
      r#"{}VALUE "FileDescription", "{}""#,
      pad.repeat(3),
      file_name
    );
    let _ = writeln!(
      string,
      r#"{}VALUE "OriginalFilename", "{}.exe""#,
      pad.repeat(3),
      file_name
    );

    let date = get_date(year);

    let _ = writeln!(
      string,
      r#"{}VALUE "LegalCopyright", "Copyright © {} {}""#,
      pad.repeat(3),
      date,
      author
    );

    let _ = writeln!(string, "{}}}", pad.repeat(2));
    let _ = write!(string, "{}}}", &pad);
  }

  fn write_var_info(string: &mut String) {
    let pad = " ".repeat(2);

    string.push_str("\n\n");

    let _ = write!(string, "{}", &pad);

    string.push_str(r#"BLOCK "VarFileInfo" {"#);
    string.push('\n');

    // Language = 0
    let _ = write!(
      string,
      r#"{}VALUE "Translation", {:#x}, 0x04b0"#,
      pad.repeat(2),
      0
    );

    let _ = write!(string, "\n{}}}", &pad);
  }

  fn parse_cargo() -> (String, String, u32) {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let contents = fs::read_to_string(file).unwrap();

    let parsed = contents.parse::<toml::Value>().unwrap();
    let pkg = parsed.get("package").unwrap();
    let metadata = pkg.get("metadata").unwrap();
    let table = metadata.as_table().unwrap();
    let name = table.get("file-name").unwrap().as_str().unwrap().to_owned();
    let author = table.get("author").unwrap().as_str().unwrap().to_owned();
    let year = table
      .get("created-year")
      .unwrap()
      .as_str()
      .unwrap()
      .parse::<u32>()
      .unwrap();

    (name, author, year)
  }

  fn get_date(created_year: u32) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    const EPOCH_START: u32 = 1970;

    let epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

    let days = epoch / (1000 * 60 * 60 * 24);
    let years = (days as f64 / 365.25) as u32;
    let current_year = EPOCH_START + years;

    let mut string = created_year.to_string();

    if created_year != current_year {
      string.push('-');
      string.push_str(&current_year.to_string());
    }

    string
  }
}

#[cfg(not(windows))]
mod build {
  pub fn generate() {}
}
