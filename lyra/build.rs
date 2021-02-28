use std::process::Command;

fn main() {
  println!("cargo:rerun-if-changed=src-js");
  println!("cargo:rerun-if-changed=src");

  println!("Formatting Rust...");
  let status = Command::new("cargo").arg("fmt").status().unwrap();
  if !status.success() {
    panic!("Build failed");
  }

  println!("Verifying JS Install...");
  let status = Command::new("yarn")
    .current_dir("./src-js")
    .arg("install")
    .status()
    .unwrap();
  if !status.success() {
    panic!("Build failed");
  }

  println!("Running Prettier...");
  let status = Command::new("yarn")
    .current_dir("./src-js")
    .arg("format")
    .status()
    .unwrap();
  if !status.success() {
    panic!("Build failed");
  }

  println!("Running Rollup...");
  let status = Command::new("yarn")
    .current_dir("./src-js")
    .arg("build")
    .status()
    .unwrap();
  if !status.success() {
    panic!("Build failed");
  }
}
