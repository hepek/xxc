extern crate clap;
use clap::App;
use std::io::{BufReader, Read, Write};
use std::fs::File;

struct Config {
    zero:  u8,
    digit: u8, // Ascii digit
    text:  u8,  // Ascii char
    ctrl:  u8,  // Ascii control char
    other: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config { zero: 8, digit: 2, text: 3, ctrl: 'a' as u8, other: 'x' as u8 }
    }
}

fn main() -> Result<(), std::io::Error> {
    let args = App::new("auto_color")
        .version("0.0.1")
        .about("colorizes hex output with default rules")
        .args_from_usage("<INPUT>             'input file'")
        .get_matches();

    let cfg: Config = Default::default();

    let f = File::open(args.value_of("INPUT").unwrap())?;
    let mut reader = BufReader::new(f);
    let mut buf = [0u8; 4096];
    let mut colors = [0u8; 4096];

    loop {
        match reader.read(&mut buf)? {
            0 => break,
            len => {
                for (b, color) in (&buf[..len]).iter().zip(&mut colors[..len]) {
                    if *b == 0u8 {
                        *color = cfg.zero;
                    } else if b.is_ascii() {
                        if b.is_ascii_control() {
                            *color = cfg.ctrl;
                        } else if b.is_ascii_digit() {
                            *color = cfg.digit;
                        } else {
                            *color = cfg.text;
                        }
                    } else {
                        *color = cfg.other;
                    }
                }

                std::io::stdout().write_all(&colors[..len])?;
            },
        }
    }

    Ok(())
}
