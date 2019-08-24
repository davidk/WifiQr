# WifiQr

This Rust crate encodes Wifi credentials into a QR code. There's a command-line interface for testing and basic use.

### Get it

Download a built binary from the releases tab.

### Usage

	WifiQR 0.01
	davidk
	Encode your wi-fi credentials as a scannable QR code

	USAGE:
	    wifiqr --ssid [ssid] --password [password] --encr [encryption type (default:wpa2)] --imagefile [output_name.png] | --svg | --svgfile [output_name.svg]

	FLAGS:
	        --hidden     Optional: Indicate whether or not the SSID is hidden
	        --svg        Emit the QR code as an SVG (to standard output)
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

This requires a complete Rust toolchain. [Link to installation instructions](https://www.rust-lang.org/tools/install).

```bash
cargo build --release
```

### Information on QR codes as used in WI-FI authentication

* [Format documentation, from zxing/zxing](https://github.com/zxing/zxing/wiki/Barcode-Contents)

### Crates used

* [qrcodegen, via project nayuki](https://docs.rs/crate/qrcodegen/1.4.0)
