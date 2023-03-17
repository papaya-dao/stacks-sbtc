use std::env::vars;

pub fn show_env() {
    vars()
        .filter(|var| var.0.contains("MINEDOC"))
        .for_each(|var| println!("{}={}", var.0, var.1));
}
