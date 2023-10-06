use std::process::Command;

fn main() {
    Command::new("npx")
        .arg("tailwindcss")
        .arg("-i")
        .arg("input.css")
        .arg("-o")
        .arg("output.css")
        .output()
        .expect("Failed to get tailwind css");
}
