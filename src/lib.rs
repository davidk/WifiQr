/// wifiqr
/// A crate to transform Wifi credentials into a scannable QR code
extern crate image;
extern crate qrcodegen;

mod exporters;

macro_rules! wifi_auth {
    // Derived from:
    // https://github.com/zxing/zxing/wiki/Barcode-Contents#wifi-network-config-android
    //
    // T: authentication type (WEP, WPA, 'nopass'). Can be ommitted for no password.
    // S: network SSID
    // P: wifi password. Can be ommitted if T is 'nopass'
    // H: Hidden SSID. Optional.
    (hidden) => {
        "WIFI:T:{};S:{};P:{};H:{};;"
    };
    (nopass) => {
        "WIFI:T:nopass;S:{};;"
    };
    (nopass_hidden) => {
        "WIFI:T:nopass;S:{};H:{};;"
    };
    () => {
        "WIFI:T:{};S:{};P:{};;"
    };
}

#[cfg(test)]
mod tests {
    use super::code::Credentials;
    use super::code::{encode, make_svg, manual_encode};
    use qrcodegen::{QrCodeEcc, Version};

    /// Basic functionality test
    #[test]
    fn test_credentials() {
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("WPA2"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:WPA2;S:test;P:password;;"
        );
    }

    /// Test credential escaping; per Zxing guidelines on how to format a `WIFI:` string
    #[test]
    fn test_credentials_escapes() {
        assert_eq!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some("randompassword"),
                Some("wpa2"),
                false,
                false
            )
            .format()
            .unwrap(),
            r###"WIFI:T:WPA2;S:\"foo\;bar\\baz\";P:randompassword;;"###
        );
    }

    /// Exercise the automatic qr encoder against the manual encoder
    #[test]
    fn test_qrcodes() {
        let credentials = Credentials::new(Some("test"), Some("WPA"), Some("test"), false, false);

        assert_eq!(
            make_svg(&encode(&credentials).unwrap()),
            make_svg(&manual_encode(
                &credentials,
                QrCodeEcc::High,
                Version::new(2),
                Version::new(15),
                None,
            ))
        );
    }

    /// Ensure that the hidden flag is added if requested
    #[test]
    fn test_hidden_ssid() {
        assert_eq!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some("randompassword"),
                Some("WPA2"),
                true,
                false
            )
            .format()
            .unwrap(),
            r###"WIFI:T:WPA2;S:\"foo\;bar\\baz\";P:randompassword;H:true;;"###
        );
    }

    /// If a ssid isn't hidden, it shouldn't be set in the formatted string
    #[test]
    fn test_normal_ssid() {
        assert_eq!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some("randompassword"),
                Some("WPA2"),
                false,
                false
            )
            .format()
            .unwrap(),
            r###"WIFI:T:WPA2;S:\"foo\;bar\\baz\";P:randompassword;;"###
        );
    }

    /// Require a password when wpa/wpa2 is requested
    #[test]
    fn test_nopassword_with_wpa2() {
        assert!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some(""),
                Some("wpa"),
                false,
                false
            )
            .format()
            .is_err(),
            "wpa2 requires a password"
        );

        assert!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some(""),
                Some("wpa2"),
                false,
                false
            )
            .format()
            .is_err(),
            "wpa2 requires a password"
        );
    }

    /// Require a password when using wep
    #[test]
    fn test_nopassword_with_wep() {
        assert!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some(""),
                Some("wep"),
                false,
                false
            )
            .format()
            .is_err(),
            "wep requires a password"
        );
    }

    #[test]
    fn test_nopassword_with_nopassword() {
        assert!(
            Credentials::new(Some("bane"), Some(""), Some("nopass"), false, false)
                .format()
                .is_ok(),
            "nopass specified with a blank password should work"
        );
    }

    /// Test various auth (T) types, like WPA/WPA2
    #[test]
    fn test_auth_types() {
        // wep
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wep"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:WEP;S:test;P:password;;"
        );

        // wpa
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("WPA"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:WPA;S:test;P:password;;"
        );

        // wpa2 -- note that the wifi string has WPA2 in caps. it seems that iOS devices are sensitive
        // to the T: parameter being lowercase (and will return 'no usable data found')
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wpa2"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:WPA2;S:test;P:password;;"
        );

        // wpa3
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wpa3"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:WPA3;S:test;P:password;;"
        );

        // nopass -- unlike wpa2/wpa3, etc, nopass is accepted by iOS devices uncapitalized
        assert_eq!(
            Credentials::new(Some("test"), Some(""), Some("nopass"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:nopass;S:test;;"
        );
    }

    #[test]
    fn test_empty_passwords_with_nopass_encr() {
        assert!(
            Credentials::new(
                Some(r###""foo;bar\baz""###),
                Some("password"),
                Some("nopass"),
                false,
                false
            )
            .format()
            .is_err(),
            "nopass cannot be specified with a password"
        );
    }

    /// ensure that nopass is set along with an empty password when it is requested by the user
    #[test]
    fn test_encr_nopass_with_empty_password() {
        assert_eq!(
            Credentials::new(Some("test"), Some(""), Some("nopass"), false, false)
                .format()
                .unwrap(),
            "WIFI:T:nopass;S:test;;"
        );
    }

    /// when quote is set, ensure that the result is quoted
    #[test]
    fn test_quoted_ssid_password() {
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wpa2"), false, true)
                .format()
                .unwrap(),
            "WIFI:T:WPA2;S:\"test\";P:\"password\";;"
        );
    }
}

/// Wifi QR code generator
pub mod code {
    use std::error;

    use image::{ImageBuffer, LumaA};
    use qrcodegen::{Mask, QrCode, QrCodeEcc, QrSegment};

    use crate::exporters::methods::{
        make_image as make_image_export, save_image as save_image_export,
        to_svg_string as to_svg_string_export,
    };

    #[derive(Debug)]
    pub struct Credentials {
        pub ssid: String,
        pub pass: String,
        pub encr: String,
        pub hidden: bool,
        pub quote: bool,
    }

    impl Credentials {
        pub fn new(
            mut _ssid: Option<&str>,
            mut _password: Option<&str>,
            mut _encr: Option<&str>,
            mut _hidden: bool,
            mut _quote: bool,
        ) -> Self {
            Credentials {
                ssid: _ssid.unwrap().to_string(),
                encr: _encr.unwrap().to_string(),
                pass: _password.unwrap().to_string(),
                hidden: _hidden,
                quote: _quote,
            }
        }

        /// escape characters as in:
        /// https://github.com/zxing/zxing/wiki/Barcode-Contents#wifi-network-config-android
        /// Special characters `\`, `;`, `,` and `:` should be escaped with a backslash
        fn filter_credentials(&self, field: &str) -> String {
            // N.B. If performance problems ever crop up, this might be more performant
            // with regex replace_all

            let mut filtered = field
                .to_string()
                .replace(r#"\"#, r#"\\"#)
                .replace('"', r#"\""#)
                .replace(r#";"#, r#"\;"#)
                .replace(r#"':'"#, r#"\:"#);

            if (filtered == self.ssid || filtered == self.pass) && self.quote {
                // println!("Adding quotes to SSID/Password -- quote is not set");
                filtered = format!("\"{}\"", field);
            }

            filtered
        }

        /// the encryption field in the Wifi QR code fails on iOS devices if it is
        /// not provided in an uppercase format. Android devices are case insensitive,
        /// so the encryption field is passed through as uppercase now.
        fn filter_encr(&self, field: &str) -> String {
            if field != "nopass" && !self.encr.is_empty() {
                return field.to_string().to_uppercase();
            }
            field.to_string()
        }

        /// Call the wifi_auth! macro to generate a qr-string and/or return any errors that
        /// need to be raised to the caller. Note: format does not enforce an encryption type, it is
        /// up to the end user to use the right value if one is provided.
        pub fn format(&self) -> Result<String, FormatError> {
            // empty password ->
            //  * is password empty and ssid hidden? => set T:nopass and H:
            //  * is encryption type empty? => set nopass
            //  * hidden ssid? => add H:
            // plain format
            // unrecoverable errors:
            // * ssid has no password, but sets a T type
            // * sets a password, but sets T type to nopass
            if (self.encr == "nopass" || self.encr.is_empty()) && !self.pass.is_empty() {
                return Err(FormatError(
                    "With nopass as the encryption type (or unset encryption type), 
                    the password field should be empty. (Encryption should probably be set 
                    to something like wpa2)".to_string(),
                ));
            }

            if self.pass.is_empty() {
                // Error condition: Password is empty, and the T (encr) type is not "nopass" / not empty
                if self.encr != "nopass" && !self.encr.is_empty() {
                    return Err(FormatError("The encryption method requested requires a password.".to_string()));
                }

                if self.encr.is_empty() || self.encr == "nopass" {
                    if self.hidden {
                        return Ok(format!(
                            wifi_auth!(nopass_hidden),
                            self.filter_credentials(&self.ssid),
                            &self.hidden,
                        ));
                    } else if self.pass.is_empty() {
                        return Ok(format!(
                            wifi_auth!(nopass),
                            self.filter_credentials(&self.ssid),
                        ));
                    }
                }
            }

            if self.hidden {
                return Ok(format!(
                    wifi_auth!(hidden),
                    self.filter_credentials(&self.filter_encr(&self.encr)),
                    self.filter_credentials(&self.ssid),
                    self.filter_credentials(&self.pass),
                    &self.hidden,
                ));
            } else {
                return Ok(format!(
                    wifi_auth!(),
                    self.filter_credentials(&self.filter_encr(&self.encr)),
                    self.filter_credentials(&self.ssid),
                    self.filter_credentials(&self.pass)
                ));
            }
        }
    }

    /// returns a new Credentials struct given Wifi credentials. This data is not validated,
    /// nor formatted into a QR code string. Call .format() on Credentials to do this.
    pub fn auth(
        _ssid: Option<&str>,
        _password: Option<&str>,
        _encr: Option<&str>,
        _hidden: bool,
        _quote: bool,
    ) -> Credentials {
        self::Credentials::new(_ssid, _password, _encr, _hidden, _quote)
    }

    /// generates a qrcode from a Credentials configuration
    pub fn encode(config: &Credentials) -> Result<QrCode, Box<dyn error::Error>> {
        let c = match config.format() {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };

        match QrCode::encode_text(&c, QrCodeEcc::High) {
            Ok(qr) => Ok(qr),
            Err(e) => Err(e.into()),
        }
    }

    /// manual_encode isn't intended for use externally, but exists to compare between the
    /// automated encoder and this manual_encode version
    /// https://docs.rs/qrcodegen/latest/src/qrcodegen/lib.rs.html#151
    pub fn manual_encode(
        config: &Credentials,
        error_level: QrCodeEcc,
        lowest_version: qrcodegen::Version,
        highest_version: qrcodegen::Version,
        mask_level: Option<Mask>,
    ) -> QrCode {
        let segs: Vec<QrSegment> = QrSegment::make_segments(&config.format().unwrap());

        QrCode::encode_segments_advanced(
            &segs,
            error_level,
            lowest_version,
            highest_version,
            mask_level,
            true,
        )
        .unwrap()
    }

    /// generates a wifi qr code that is printed to a terminal/console for quick scanning
    /// parameters:
    /// - qrcode: encoded qrcode
    /// - quiet_zone: the border size to apply to the QR code (created with ASCII_BL_BLOCK)
    /// result:
    /// - this prints a block of text directly to the console
    pub fn console_qr(qrcode: &QrCode, quiet_zone: i32) {
        const ASCII_BL_BLOCK: &str = "  ";
        const ASCII_W_BLOCK: &str = "██";

        let x_zone = quiet_zone;
        let y_zone = quiet_zone;

        // paint top border -- y axis
        for _top_border in 0..y_zone {
            print!("{}", ASCII_BL_BLOCK);
            println!();
        }

        for y in 0..qrcode.size() {
            // paint left border -- x axis
            for _left_border in 0..x_zone {
                print!("{}", ASCII_BL_BLOCK);
            }

            // paint qr
            for x in 0..qrcode.size() {
                if qrcode.get_module(x, y) {
                    print!("{}", ASCII_W_BLOCK);
                } else {
                    print!("{}", ASCII_BL_BLOCK);
                }
            }

            // paint right border -- x axis
            for _right_border in 0..x_zone {
                print!("{}", ASCII_BL_BLOCK);
            }

            println!();
        }

        // paint bottom border -- y axis
        for _bottom_border in 0..y_zone {
            print!("{}", ASCII_BL_BLOCK);
            println!();
        }
    }

    pub fn make_image(
        qrcode: &QrCode,
        scale: i32,
        border_size: i32,
    ) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
        make_image_export(qrcode, scale, border_size)
    }

    /// generates an svg string from a QrCode (output from the QR library)
    ///
    /// * qrcode: &QrCode
    ///
    pub fn make_svg(qrcode: &QrCode) -> String {
        to_svg_string_export(qrcode, 4)
    }

    /// saves an image to a file
    ///
    /// * image: ImageBuffer<>
    ///
    /// * save_file: file path to save the image into
    pub fn save_image(
        image: &ImageBuffer<LumaA<u8>, Vec<u8>>,
        save_file: String,
    ) -> Result<(), image::ImageError> {
        save_image_export(image, save_file)
    }

    /// this error is returned when a potentally invalid combination of choices are made in the process
    /// of building a wifi connection string to embed as a QR code.
    ///
    /// a recommendation is returned to the caller as a string to provide corrective action
    #[derive(Debug, Clone)]
    pub struct FormatError(String);

    impl std::error::Error for FormatError {
        fn description(&self) -> &str {
            &self.0
        }
    }

    impl std::fmt::Display for FormatError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

}