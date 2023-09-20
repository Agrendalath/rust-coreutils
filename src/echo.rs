// Implement the fish-shell version of echo.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Do not output a newline.
    #[arg(short, default_value_t = false)]
    n: bool,

    /// Do not separate arguments with spaces.
    #[arg(short, default_value_t = false)]
    s: bool,

    // Ignore the -E option, as it is the default behaviour.
    // Also, clap does not support multiple short options for a single argument.
    /// Enable interpretation of backslash escapes.
    /// The following sequences are recognized:
    ///   • \ backslash
    ///   • \a alert (BEL)
    ///   • \b backspace
    ///   • \c produce no further output
    ///   • \e escape
    ///   • \f form feed
    ///   • \n new line
    ///   • \r carriage return
    ///   • \t horizontal tab
    ///   • \v vertical tab
    ///   • \0NNN byte with octal value NNN (1 to 3 digits)
    ///   • \xHH byte with hexadecimal value HH (1 to 2 digits)
    #[arg(short, default_value_t = false, verbatim_doc_comment)]
    e: bool,

    data: Vec<String>,
}

pub fn main() {
    let args = Args::parse();

    let data = echo(args);

    print!("{data}");
}

fn echo(args: Args) -> String {
    let line_ending = if args.n { "" } else { "\n" };
    let arguments_join = if args.s { "" } else { " " };

    let mut data = args.data.join(arguments_join) + line_ending;
    if args.e {
        data = data.replace("\\\\", "\\");
        data = data.replace("\\a", "\x07");
        data = data.replace("\\b", "\x08");

        let truncate_offset = data.find("\\c").unwrap_or(data.len());
        data.truncate(truncate_offset);

        data = data.replace("\\e", "\x1b");
        data = data.replace("\\f", "\x0c");
        data = data.replace("\\n", "\n");
        data = data.replace("\\r", "\r");
        data = data.replace("\\t", "\t");
        data = data.replace("\\v", "\x0b");

        // Use regex to find \0NNN and \xHH.
        let octals = regex::Regex::new(r"\\0[0-7]{1,3}").unwrap();
        data = octals
            .replace_all(&data, |caps: &regex::Captures| {
                let octal = caps.get(0).unwrap().as_str();
                let octal = &octal[2..];
                let octal = u8::from_str_radix(octal, 8).unwrap();
                let octal = std::char::from_u32(octal as u32).unwrap();
                octal.to_string()
            })
            .to_string();
        let hexadecimals = regex::Regex::new(r"\\x[0-9a-fA-F]{1,2}").unwrap();
        data = hexadecimals
            .replace_all(&data, |caps: &regex::Captures| {
                let hexadecimal = caps.get(0).unwrap().as_str();
                let hexadecimal = &hexadecimal[2..];
                let hexadecimal = u8::from_str_radix(hexadecimal, 16).unwrap();
                let hexadecimal = std::char::from_u32(hexadecimal as u32).unwrap();
                hexadecimal.to_string()
            })
            .to_string();
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn test_binary() {
        let mut cmd = Command::cargo_bin("echo").unwrap();
        cmd.assert().success().stdout("\n");

        let mut cmd = Command::cargo_bin("echo").unwrap();
        cmd.arg("-nse")
            .arg("data")
            .arg(r"more \ndata")
            .assert()
            .success()
            .stdout("datamore \ndata");
    }

    #[test]
    fn test_no_args() {
        let args = Args::parse_from(&["echo"]);
        assert_eq!(echo(args), "\n");

        let args = Args::parse_from(&["echo", "data"]);
        assert_eq!(echo(args), "data\n");

        let args = Args::parse_from(&["echo", "data", "more data"]);
        assert_eq!(echo(args), "data more data\n");

        let args = Args::parse_from(&["echo", "data", "more data\\n"]);
        assert_eq!(echo(args), "data more data\\n\n");
    }

    #[test]
    fn test_n() {
        let args = Args::parse_from(&["echo", "-n"]);
        assert_eq!(echo(args), "");

        let args = Args::parse_from(&["echo", "-n", "data"]);
        assert_eq!(echo(args), "data");

        let args = Args::parse_from(&["echo", "-n", "data", "more data"]);
        assert_eq!(echo(args), "data more data");

        let args = Args::parse_from(&["echo", "-n", "data\\n"]);
        assert_eq!(echo(args), "data\\n");
    }

    #[test]
    fn test_s() {
        let args = Args::parse_from(&["echo", "-s"]);
        assert_eq!(echo(args), "\n");

        let args = Args::parse_from(&["echo", "-s", "data"]);
        assert_eq!(echo(args), "data\n");

        let args = Args::parse_from(&["echo", "-s", "data", "more data"]);
        assert_eq!(echo(args), "datamore data\n");

        let args = Args::parse_from(&["echo", "-s", "data\\n"]);
        assert_eq!(echo(args), "data\\n\n");
    }

    #[test]
    fn test_e() {
        let args = Args::parse_from(&["echo", "-e"]);
        assert_eq!(echo(args), "\n");

        let args = Args::parse_from(&["echo", "-e", "data"]);
        assert_eq!(echo(args), "data\n");

        let args = Args::parse_from(&["echo", "-e", r"data\\", "more data"]);
        assert_eq!(echo(args), "data\\ more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data\a", "more data"]);
        assert_eq!(echo(args), "data\x07 more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data\b", "more data"]);
        assert_eq!(echo(args), "data\x08 more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \c more data"]);
        assert_eq!(echo(args), "data ");

        let args = Args::parse_from(&["echo", "-e", r"data\e more data"]);
        assert_eq!(echo(args), "data\x1b more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \f more data"]);
        assert_eq!(echo(args), "data \x0c more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \n more data"]);
        assert_eq!(echo(args), "data \n more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data\n"]);
        assert_eq!(echo(args), "data\n\n");

        let args = Args::parse_from(&["echo", "-e", r"data \r more data"]);
        assert_eq!(echo(args), "data \r more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \t more data"]);
        assert_eq!(echo(args), "data \t more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \v more data"]);
        assert_eq!(echo(args), "data \x0b more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \0153 more data"]);
        assert_eq!(echo(args), "data k more data\n");

        let args = Args::parse_from(&["echo", "-e", r"data \x75 more data"]);
        assert_eq!(echo(args), "data u more data\n");
    }
    #[test]
    fn test_nse() {
        let args = Args::parse_from(&["echo", "-n", "-s", "-e"]);
        assert_eq!(echo(args), "");

        let args = Args::parse_from(&["echo", "-nse", "data"]);
        assert_eq!(echo(args), "data");

        let args = Args::parse_from(&["echo", "-nse", "data", r"more \ndata"]);
        assert_eq!(echo(args), "datamore \ndata");
    }
}
