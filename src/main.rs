extern crate clap;
extern crate rpassword;
extern crate wifiqr;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use clap::{App, Arg, ArgGroup};
use rpassword::prompt_password_stdout;

fn main() {
    let options = App::new("WifiQR")
        .version("0.0.8")
        .about("Encode your wi-fi credentials as a scannable QR code")
        .author("davidk")
        .usage("wifiqr [ --ssid (ssid) ] [ --password (password) | --ask ]
            [ --encr encryption type (default:wpa2) ]
            [ --imagefile (output_name.png) | --svg | --svgfile (output_name.svg) ]")
        .arg(
            Arg::with_name("ssid")
                .long("ssid")
                .takes_value(true)
                .required(true)
                .display_order(1)
                .help("Sets the WiFi SSID"),
        )
        .arg(
            Arg::with_name("password")
                .long("password")
                .takes_value(true)
                .default_value("")
                .display_order(2)
                .help("Sets the WiFi password"),
        )
        .arg(
            Arg::with_name("encryption")
                .long("encr")
                .takes_value(true)
                .default_value("wpa2")
                .display_order(3)
                .help("The WiFi's encryption type (wpa, wpa2, nopass)"),
        )
        .arg(
            Arg::with_name("hidden")
                .long("hidden")
                .display_order(4)
                .takes_value(false)
                .help("Optional: Indicate whether or not the SSID is hidden"),
        )
        .arg(
            Arg::with_name("scale")
                .long("scale")
                .takes_value(true)
                .default_value("10")
                .display_order(5)
                .help("QR code scaling factor"),
        )
        .arg(
            Arg::with_name("quiet_zone")
                .long("quietzone")
                .takes_value(true)
                .display_order(6)
                .default_value("2")
                .help("QR code: The size of the quiet zone/border to apply to the final QR code"),
        )
        .arg(
            Arg::with_name("image_file")
                .long("imagefile")
                .takes_value(true)
                .display_order(7)
                .help("The name of the file to save to (e.g. --imagefile qr.png). Formats: [png, jpg, bmp]"),
        )
        .arg(
            Arg::with_name("svg")
                .long("svg")
                .takes_value(false)
                .display_order(8)
                .help("Emit the QR code as an SVG (to standard output)")
        )
        .arg(
            Arg::with_name("svg_file")
                .long("svgfile")
                .takes_value(true)
                .display_order(9)
                .help("Save the QR code to a file (SVG formatted)")
        )
        .arg(
            Arg::with_name("console")
                .long("console")
                .display_order(10)
                .help("Print the QR code out to the console")
        )
        .group(
            ArgGroup::with_name("output types")
                .required(true)
                .args(&["image_file","svg","svg_file","console"])
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .takes_value(false)
                .display_order(11)
                .help("Display some extra debugging output")
        )
        .arg(
            Arg::with_name("ask")
                .long("ask")
                .short("a")
                .takes_value(false)
                .display_order(12)
                .help("Ask for password instead of getting it through the command-line")
        )
        .arg(
            Arg::with_name("ask-echo")
                .long("ask-echo")
                .takes_value(false)
                .display_order(13)
                .help("Ask for password while displaying input on the console")
        )
        .arg(
            Arg::with_name("quote")
                .long("quote")
                .takes_value(false)
                .display_order(14)
                .help("If the SSID or password could be mistaken for a hexadecimal value, 
                    this option will add double-quotes around the SSID and password")
        )
        .get_matches();

    // Note: avoid turbofish/generic on parse() through upfront declaration
    let scale: i32 = options.value_of("scale").unwrap_or("10").parse().unwrap();
    let quiet_zone: i32 = options
        .value_of("quiet_zone")
        .unwrap_or("10")
        .parse()
        .unwrap();
    let image_file: String = options
        .value_of("image_file")
        .unwrap_or("")
        .parse()
        .unwrap();

    // Validate that image_file extension supplied is compatible with upstream library export formats
    match Path::new(&image_file).extension() {
        None => {
            println!("Error: No extension found for image file. Try --imagefile [ qr.jpeg | qr.png ] instead.");
            return;
        }
        Some(p) => {
            let ext: &str = p.to_str().unwrap();
            match ext {
                "png" | "jpeg" | "jpg" => {}
                _ => {
                    println!("Unrecognized file extension: {}. Try --imagefile [ qr.png | qr.jpeg | qr.jpg ] instead.", ext);
                    return;
                }
            }
        }
    };

    let mut password = String::new();

    if options.is_present("ask") {
        password = prompt_password_stdout(
            format!(
                "Enter password for network `{}` (will not echo to screen): ",
                options.value_of("ssid").unwrap()
            )
            .as_str(),
        )
        .unwrap();
    } else if options.is_present("ask-echo") {
        print!(
            "Enter password for network `{}` (will echo to screen): ",
            options.value_of("ssid").unwrap()
        );

        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut password)
            .expect("Failed to read password");
        password = password.trim().to_string();
    } else {
        password = options.value_of("password").unwrap().to_string();
    }

    let config = wifiqr::code::auth(
        options.value_of("ssid"),
        Some(&password),
        options.value_of("encryption"),
        options.is_present("hidden"),
        options.is_present("quote"),
    );

    if options.is_present("debug") {
        println!(
            "SSID: {} | PASSWORD: {} | ENCRYPTION: {} | HIDDEN: {} | QUOTE SSID/PASSWORD: {}",
            options.value_of("ssid").unwrap(),
            password,
            options.value_of("encryption").unwrap(),
            options.is_present("hidden"),
            options.is_present("quote"),
        );

        println!("Wifi string: {:?}", config.format().unwrap());
    }

    let encoding = match wifiqr::code::encode(&config) {
        Ok(e) => e,
        Err(e) => {
            println!("There was a problem generating the QR code.\n{}", e);
            return;
        }
    };

    if options.is_present("svg_file") {
        println!("Generating QR code ..");
        let file_name = options.value_of("svg_file").unwrap();

        println!("Writing out to SVG file: {} ..", file_name);
        let svg_data = wifiqr::code::make_svg(&encoding);

        fs::write(file_name, svg_data).expect("Unable to write file");
    } else if options.is_present("image_file") {
        println!("Generating QR code ..");

        println!("Parameters: scale {} + quiet zone: {} ", scale, quiet_zone);

        let image = wifiqr::code::make_image(&encoding, scale, quiet_zone);
        match wifiqr::code::save_image(&image, image_file.to_string()) {
            Ok(_) => {
                println!("QR code has been saved to file {}", image_file);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                println!("Unable to write QR image to file in requested format. Supported extensions are .jpeg and .png. Try --imagefile qr.jpeg or --imagefile qr.png");
                fs::remove_file(image_file.to_string()).unwrap();
            }
        };
    } else if options.is_present("svg") {
        println!("{}", wifiqr::code::make_svg(&encoding));
    } else if options.is_present("console") {
        wifiqr::code::console_qr(&encoding, quiet_zone);
    } else {
        println!("Please select an output format. For available formats, re-run with --help");
    }
}
