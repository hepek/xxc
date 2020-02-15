extern crate clap;
use clap::App;
use std::io::{BufReader, Write};
use std::fs::File;
mod reader;

fn main() -> Result<(), std::io::Error> {
    let args = App::new("mpegts_color - colorizes hex output with mpegts rules")
        .version("0.0.1")
        .author("Milan Markovic <zivotinja@gmail.com>")
        .args_from_usage("<INPUT>  'input file'")
        .get_matches();

    let f = File::open(args.value_of("INPUT").unwrap())?;
    let mut reader = BufReader::new(f);
    let mut buf = [0u8; 21*188];
    let mut colors = [0u8; 21*188];
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    loop {
        match reader::read_fill(&mut reader, &mut buf)? {
            0 => break,
            len => {
                for (ts_packet, color) in (&buf[..len]).chunks(188).zip(&mut colors[..len].chunks_mut(188)) {
                    let clr = colorize_packet(ts_packet);
                    if clr.len() == color.len() {
                        color.copy_from_slice(&clr);
                    } else {
                        for (dst, src) in color.iter_mut().zip(clr.iter()) {
                            *dst = *src;
                        }
                    }
                }

                handle.write_all(&colors[..len])?;
            },
        }
    }

    Ok(())
}

fn colorize_packet(buf: &[u8]) -> [u8; 188] {
    let mut res = ['y' as u8; 188];

    if buf[0] == 0x47 {
        res[0] = 'd' as u8;
    } else {
        res[0] = 'Y' as u8;
    }

    res[1] = 'a' as u8;
    res[2] = 'a' as u8;
    res[3] = 'a' as u8;
    let mut adaptation = 4;
    let mut data = 4;

    match buf[3] & 0b00110000 {
        0b00110000 => { // both data and adapt
            let adlen = buf[4];
            adaptation = 5 + (adlen as usize);
            data = 188;
        },
        0b00010000 => { // data only
            data = 188;
        },
        0b00100000 => { // adaptation only
            let adlen = buf[4];
            adaptation = 5 + (adlen as usize);
            data = adaptation;
        },
        _ => {
        }
    }

    if adaptation > 188 {
        adaptation = 188;
    }

    if data > 188 {
        data = 188;
    }

    for clr in res[4..adaptation].iter_mut() {
        *clr = 'y' as u8; 
    }

    for clr in res[adaptation..data].iter_mut() {
        *clr = 't' as u8;
    }

    for clr in res[data..].iter_mut() {
        *clr = 'f' as u8;
    }

    res
}
