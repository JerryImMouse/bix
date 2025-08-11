use std::{fs::{File, OpenOptions}, io::{Read, Seek, Write}, path::PathBuf};
use clap::{Parser, Subcommand};

type BixResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Parser, Debug)]
#[command(about, long_about = None, version)]
#[command(next_line_help = true)]
pub struct Args {
    #[command(subcommand)]
    command: ArgCommand
}

#[derive(Subcommand, Debug)]
pub enum ArgCommand {
    /// Prints out hexadecimal representation of a specified file
    View {
        file: PathBuf,

        /// Hexadecimal offset in the file, e.g 0x10 
        #[arg(short, long, default_value = "0x0", value_parser = parse_offset)]
        offset: u64,

        /// Number of bytes to output since `offset`, e.g -o 0x0 -n 256 will output 256 bytes since
        /// 0x0
        #[arg(short, long)]
        number: Option<usize>,

        /// Number of bytes per row
        #[arg(short, long, default_value_t = 16)]
        width: usize,

        /// Should we print extra whitespace between 2 parts of the bytes, like 8b-2whitespace-8b
        #[arg(long)]
        no_group: bool,

        /// Should we print address rows
        #[arg(long)]
        no_addr: bool,

        /// Should we print ASCII representation of bytes in a row
        #[arg(long)]
        no_ascii: bool,

        /// Same as using `--no-addr --no-group --no-ascii` together
        #[arg(long)]
        raw: bool,
    },
    /// Writes N bytes at the specified offset into a specified file
    Set {
        file: PathBuf,
        
        /// Array of bytes in hex format to insert. E.g `bix set filename AA DD CC BA`
        #[arg(required = true, value_parser = parse_byte)]
        bytes: Vec<u8>,

        /// Hexadecimal offset in the file, e.g 0x10
        #[arg(short, long, default_value = "0x0", value_parser = parse_offset)]
        offset: u64,
    }
}

fn parse_byte(s: &str) -> BixResult<u8> {
    Ok(u8::from_str_radix(s, 16)?)
}

fn parse_offset(s: &str) -> BixResult<u64> {
    if let Some(stripped) = s.strip_prefix("0x") {
        Ok(u64::from_str_radix(stripped, 16)?)
    } else {
        Ok(s.parse::<u64>()?)
    }
}

fn main() -> BixResult<()> {
    let cli = Args::parse();
    match cli.command {
        ArgCommand::View { file, offset, number, width, no_group, no_addr, no_ascii, raw } => {
            let mut f = File::open(file)?;
            f.seek(std::io::SeekFrom::Start(offset))?;

            let mut buf = Vec::new();
            if let Some(len) = number {
                buf.resize(len, 0);
                f.read_exact(&mut buf)?;
            } else {
                f.read_to_end(&mut buf)?;
            }

            if raw {
                for byte in &buf {
                    print!("{:02X} ", byte);
                }
                println!();
                return Ok(());
            }

            for (i, chunk) in buf.chunks(width).enumerate() {
                let addr = offset + (i * width) as u64;

                if !no_addr {
                    print!("{:08X}: ", addr);
                }

                let mid = width / 2;
                for (j, byte) in chunk.iter().enumerate() {
                    print!("{:02X} ", byte);
                    if !no_group && j + 1 == mid {
                        print!(" ");
                    }
                }

                if !no_ascii {
                    let hex_width = if !no_addr { width * 3 + 1 } else { width * 3 };
                    let printed_hex = chunk.len() * 3 + if chunk.len() > 8 { 1 } else { 0 };
                    print!("{:width$}|", "", width = hex_width - printed_hex);

                    for byte in chunk {
                        let ch = if (byte.is_ascii_graphic() || *byte == b' ') && !byte.is_ascii_control() {
                            *byte as char
                        } else {
                            '.'
                        };
                        print!("{}", ch);
                    }
                    print!("|");
                }

                println!();
            }

            Ok(())
        },
        ArgCommand::Set { offset, file, bytes } => {
            let mut f = OpenOptions::new()
                .write(true)
                .open(file)?;
            
            f.seek(std::io::SeekFrom::Start(offset))?;
            f.write_all(&bytes)?;
            println!("Wrote {} bytes at 0x{:X}", bytes.len(), offset);
            Ok(())
        }
    }
}
