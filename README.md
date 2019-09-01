# WifiQr

This Rust crate encodes Wifi credentials into a QR code. A command-line interface is available for testing and basic use, too.

### Releases

To use WifiQr's command-line implementation, download a pre-built binary from the releases tab.

### Crate

The crate is not currently available on crates.io, but can be downloaded from this repository. Please be sure to pin the version you're using to a specific commit/release, to avoid backwards incompatible changes (this is not anticipated, but it may happen).

### Usage

	WifiQR 0.02
	davidk
	Encode your wi-fi credentials as a scannable QR code

	USAGE:
	    wifiqr --ssid (ssid) [ --password (password) | --ask ] --encr [ encryption type (default:wpa2) ] [ --imagefile (output_name.png) | --svg | --svgfile (output_name.svg) ]

	FLAGS:
		--hidden     Optional: Indicate whether or not the SSID is hidden
		--svg        Emit the QR code as an SVG (to standard output)
		--console    Print the QR code out to the console
	    -d, --debug      Display some extra debugging output
	    -a, --ask        Ask for password instead of getting it through the command-line
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

#### Building

Pre-built releases are provided on GitHub, but for development, or to build your own from source (after installing the [Rust toolchain](https://www.rust-lang.org/tools/install):

	cargo build --release

### Information on QR codes as used in WI-FI authentication

* [Format documentation, from zxing/zxing](https://github.com/zxing/zxing/wiki/Barcode-Contents)

### Crates Used

* [qrcodegen, via project nayuki](https://docs.rs/crate/qrcodegen/1.4.0)
