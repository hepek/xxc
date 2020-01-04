extern crate clap;
use clap::App;
use std::io::{BufReader, Read, Write};
use std::fs::File;

fn main() -> Result<(), std::io::Error> {
    let args = App::new("xxc - print colorful hex")
        .version("0.0.1")
        .author("Milan Markovic <zivotinja@gmail.com>")
        .about("hex print with colors")
        .args_from_usage(
            "-c, --color=[FILE] 'sets custom coloring scheme'
            -a, --auto         'color hex by context'
            <INPUT>             'input file'")
        .get_matches();

    let f = File::open(args.value_of("INPUT").unwrap())?;
    let mut reader = BufReader::new(f);

    let res = if let Some(clr) = args.value_of("color") {
        let f = File::open(clr)?;
        let mut color_reader = BufReader::new(f);
        print(&mut reader, &mut color_reader) 
    } else if args.is_present("auto") {
        print_auto(&mut reader)
    } else {
        print(&mut reader, &mut ConstReader{ value: 15 })
    };

    match res {
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        _ => res,
    }
}

struct ConstReader {
    value: u8
}

impl Read for ConstReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
       for b in buf.iter_mut() {
           *b = self.value;
       }

       Ok(buf.len())
    }
}

fn color_auto(buf: &[u8; 16]) -> [u8; 16] {
    let mut res = [15u8; 16];

    for (b, clr) in buf.iter().zip(res.iter_mut()) {
        if b.is_ascii() {
            if b.is_ascii_control() {
                *clr = 51;
            } else if b.is_ascii_digit() {
                *clr = 50; 
            } else {
                *clr = 97;
            }
        } else {
            *clr = 15;
        }
    }

    res
}

fn print_auto(reader: &mut dyn Read) -> Result<(), std::io::Error> {
    let mut buf = [0u8; 16];
    let mut offset = 0u64;

    loop {
        match reader.read(&mut buf)?
        {
            0 => return Ok(()), // EOF
            16 => {
                print_line(offset, &buf[..], &color_auto(&buf))?;
                offset += 16;
            },
            len => print_line(offset, &buf[..len], &color_auto(&buf))?,
        }
    };
}


fn print(reader: &mut dyn Read, color_reader: &mut dyn Read) -> Result<(), std::io::Error> {
    let mut buf = [0u8; 16];
    let mut clr = [0u8; 16];
    let mut offset = 0u64;

    loop {
        match (reader.read(&mut buf)?, color_reader.read(&mut clr)?)
        {
            (0, _) => return Ok(()), // EOF
            (16, 16) => {
                print_line(offset, &buf[..], &clr[..])?;
                offset += 16;
            },
            (len1, len2) if len1 <= len2 => print_line(offset, &buf[..len1], &clr[..len1])?,
            (_, _) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "mismatch in color and input size")),
        }
    };
}


fn print_line(offset: u64, buf: &[u8], clr: &[u8]) -> Result<(), std::io::Error> {
    use colorful::{Color, Colorful};
    use std::str;

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_fmt(format_args!("{:08x}: ", offset))?;
    let mut last_color = Color::White;
    for (idx, (b, c)) in buf.iter().zip(clr.iter()).enumerate() {
        last_color = get_color(*c);
        handle.write_fmt(format_args!("{}", format!("{:02x}", b).color(last_color)))?;
        if idx % 2 == 1 {
            handle.write_fmt(format_args!("{}", " ".color(last_color)))?;
        }
    }
    for i in buf.len()..16 {
        handle.write_fmt(format_args!("{}", "  ".color(last_color)))?;

        if i % 2 == 1 {
            handle.write_fmt(format_args!("{}", " ".color(last_color)))?;
        }
    }
    handle.write_fmt(format_args!("{}", "  ".color(last_color)))?;
    for (b, c) in buf.iter().zip(clr.iter()) {
        last_color = get_color(*c);
        if b.is_ascii() && !b.is_ascii_control() { 
            let chr = [*b];
            handle.write_fmt(format_args!("{}", unsafe { str::from_utf8_unchecked(&chr[..]) }.color(last_color)))?;
        } else {
            handle.write_fmt(format_args!("{}", ".".color(last_color)))?;
        }
    }
    handle.write_all(b"\n")?;
    Ok(())
}

fn get_color(c: u8) -> colorful::Color {
    use colorful::Color;
    *Color::iterator().nth(c as usize).unwrap()
}
