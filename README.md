# WifiQr

This Rust crate encodes Wifi credentials into a QR code. A [command-line interface](https://github.com/davidk/WifiQr/releases) is available for testing and basic use, too.

<p align="center"><img src="/img/wifiqr-console.gif?raw=true"/></p>

### Releases

To use WifiQr's command-line implementation, download a pre-built binary from the [releases tab](https://github.com/davidk/WifiQr/releases).

### Utility usage

	USAGE:
	    wifiqr [ --ssid (ssid) ] [ --password (password) | --ask ]
		    [ --encr encryption type (default:wpa2) ]
		    [ --imagefile (output_name.png) | --svg | --svgfile (output_name.svg) ]

	FLAGS:
		--hidden     Optional: Indicate whether or not the SSID is hidden
		--svg        Emit the QR code as an SVG (to standard output)
		--console    Print the QR code out to the console
	    -d, --debug      Display some extra debugging output
	    -a, --ask        Ask for password instead of getting it through the command-line
		--quote      If the SSID or password could be mistaken for a hexadecimal value, 
						 this option will add double-quotes around the SSID and password
	    -h, --help       Prints help information
	    -V, --version    Prints version information

	OPTIONS:
		--ssid <ssid>               Sets the WiFi SSID
		--password <password>       Sets the WiFi password [default: ]
		--encr <encryption>         The WiFi's encryption type (wpa, wpa2, nopass) [default: wpa2]
		--scale <scale>             QR code scaling factor [default: 10]
		--quietzone <quiet_zone>    QR code: The size of the quiet zone/border to apply to the final QR code [default:
					    2]
		--imagefile <image_file>    The name of the file to save to (e.g. --imagefile qr.png). Formats: [png, jpg, bmp]
		--svgfile <svg_file>        Save the QR code to a file (SVG formatted)

### Crate

The crate is not currently available on crates.io, but can be downloaded from this repository. Please be sure to pin the version you're using to a specific commit/release, to avoid sudden backwards incompatible changes (this is not anticipated, but it may happen).

```rust
extern crate wifiqr;
let quiet_zone = 5;
let config = wifiqr::code::auth(
    ssid,				// Network name
    password,				// Network password/passphrase
    encryption,				// WPA/WPA2/WEP
    is_hidden,				// boolean
    quote_ssid_password,		// boolean
);

let encoding = wifiqr::code::encode(&config).expect("There was a problem generating the QR code");

// this prints a qr code to the console
wifiqr::code::console_qr(&encoding, quiet_zone);

// this passes the svg output from the QR encoder back
println!("{}", wifiqr::code::make_svg(&encoding));
```

#### Building

Pre-built releases are provided on GitHub, but for development, or to build your own from source (after installing the [Rust toolchain](https://www.rust-lang.org/tools/install):

	cargo build --release

### Information on QR codes as used in WI-FI authentication

* [Format documentation, from zxing/zxing](https://github.com/zxing/zxing/wiki/Barcode-Contents)

### Crates Used

* [qrcodegen, via project nayuki](https://docs.rs/crate/qrcodegen/1.4.0)
