// wifiqr
// A crate to transform Wifi credentials into a scannable QR code
//
// Current limitations:
// * Hexadecimal input for the wifi password is not supported
extern crate image;
extern crate qrcodegen;

// * HEX `S` / `P` : its possible that these could be interpreted as hex if ascii. Add quotes
// unless an option is set to ignore it
macro_rules! wifi_auth {
    // Derived from:
    // https://github.com/zxing/zxing/wiki/Barcode-Contents#wifi-network-config-android
    //
    // T: authentication type (WEP, WPA, 'nopass'). Can be ommitted for no password.
    // S: network SSID
    // P: wifi password. Can be ommitted if T is 'nopass'
    // H: Hidden SSID. Optional.
    (hidden) => ("WIFI:T:{};S:{};P:{};H:{};;");
    (nopass) => ("WIFI:T:nopass;S:{};;");
    (nopass_hidden) => ("WIFI:T:nopass;S:{};H:{};;");
    () => {
        "WIFI:T:{};S:{};P:{};;";
    };
}

#[cfg(test)]
mod tests {
    use super::code::Credentials;
    use super::code::{encode, make_svg, manual_encode};
    use qrcodegen::{QrCodeEcc, Version};

    // Basic functionality test
    #[test]
    fn test_credentials() {
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("WPA2"), false).format().unwrap(),
            "WIFI:T:WPA2;S:test;P:password;;"
        );
    }

    // Test credential escaping; per Zxing guidelines on how to format a `WIFI:` string
    #[test]
    fn test_credentials_escapes() {
        assert_eq!(
            Credentials::new(Some(r###""foo;bar\baz""###), 
                             Some("randompassword"), 
                             Some("wpa2"), 
                             false).format().unwrap(),
            r###"WIFI:T:WPA2;S:\"foo\;bar\\baz\";P:randompassword;;"###
        );
    }

    // Exercise the automatic qr encoder against the manual encoder
    #[test]
    fn test_qrcodes() {
        let credentials = Credentials::new(Some("test"), Some("WPA"), Some("test"), false);

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
   
    // Ensure that the hidden flag is added if requested 
    #[test]
    fn test_hidden_ssid() {
        assert_eq!(Credentials::new(Some(r###""foo;bar\baz""###), 
                                    Some("randompassword"), 
                                    Some("WPA2"), true).format().unwrap(),
            r###"WIFI:T:WPA2;S:\"foo\;bar\\baz\";P:randompassword;H:true;;"###);
    }

    // If a ssid isn't hidden, it shouldn't be set in the formatted string
    #[test]
    fn test_normal_ssid() {
        assert_eq!(Credentials::new(Some(r###""foo;bar\baz""###), 
                                    Some("randompassword"), 
                                    Some("WPA2"), false).format().unwrap(),
            r###"WIFI:T:WPA2;S:\"foo\;bar\\baz\";P:randompassword;;"###);
    }

    // Require a password when wpa/wpa2 is requested
    #[test]
    fn test_nopassword_with_wpa2() {
        assert!(Credentials::new(Some(r###""foo;bar\baz""###), 
                                    Some(""), 
                                    Some("wpa"), 
                                    false).format().is_err(), "wpa2 requires a password");
 
        assert!(Credentials::new(Some(r###""foo;bar\baz""###), 
                                    Some(""), 
                                    Some("wpa2"), 
                                    false).format().is_err(), "wpa2 requires a password");
    }

    // Require a password when using wep
    #[test]
    fn test_nopassword_with_wep() {
        assert!(Credentials::new(Some(r###""foo;bar\baz""###), 
                                    Some(""), 
                                    Some("wep"), 
                                    false).format().is_err(), "wep requires a password");
    }

    #[test]
    fn test_nopassword_with_nopassword() {
        assert!(Credentials::new(Some("bane"), 
                                    Some(""), 
                                    Some("nopass"), 
                                    false).format().is_ok(), "nopass specified with a blank password should work");
    }

    // Test various auth (T) types, like WPA/WPA2
    #[test]
    fn test_auth_types() {
        // wep
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wep"), false).format().unwrap(),
            "WIFI:T:WEP;S:test;P:password;;"
        );

        // wpa
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("WPA"), false).format().unwrap(),
            "WIFI:T:WPA;S:test;P:password;;"
        );

        // wpa2 -- note that the wifi string has WPA2 in caps. it seems that iOS devices are sensitive
        // to the T: parameter being lowercase (and will return 'no usable data found')
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wpa2"), false).format().unwrap(),
            "WIFI:T:WPA2;S:test;P:password;;"
        );

        // wpa3
        assert_eq!(
            Credentials::new(Some("test"), Some("password"), Some("wpa3"), false).format().unwrap(),
            "WIFI:T:WPA3;S:test;P:password;;"
        );

        // nopass -- unlike wpa2/wpa3, etc, nopass is accepted by iOS devices uncapitalized
        assert_eq!(
            Credentials::new(Some("test"), Some(""), Some("nopass"), false).format().unwrap(),
            "WIFI:T:nopass;S:test;;"
        );
    }

    #[test]
    fn test_empty_passwords_with_nopass_encr() {
        assert!(Credentials::new(Some(r###""foo;bar\baz""###), 
                                    Some("password"), 
                                    Some("nopass"), 
                                    false).format().is_err(), "nopass cannot be specified with a password");
    }

    // ensure that nopass is set along with an empty password when it is requested by the user
    #[test]
    fn test_encr_nopass_with_empty_password() {
        assert_eq!(
            Credentials::new(Some("test"), Some(""), Some("nopass"), false).format().unwrap(),
            "WIFI:T:nopass;S:test;;"
        );
    }
}

pub mod code {

    use std::convert::TryInto;

    use image::{ImageBuffer, LumaA};
    use qrcodegen::{DataTooLong, Mask, QrCode, QrCodeEcc, QrSegment};

    use imageproc::drawing::draw_filled_rect_mut;
    use imageproc::rect::Rect;

    #[derive(Debug)]
    pub struct Credentials {
        pub ssid: String,
        pub pass: String,
        pub encr: String,
        pub hidden: bool,
    }

    impl Credentials {
        pub fn new(
            mut _ssid: Option<&str>,
            mut _password: Option<&str>,
            mut _encr: Option<&str>,
            mut _hidden: bool,
        ) -> Self {
            return Credentials {
                ssid: _ssid.unwrap().to_string(),
                encr: _encr.unwrap().to_string(),
                pass: _password.unwrap().to_string(),
                hidden: _hidden
            };
        }

        // escape characters as in:
        // https://github.com/zxing/zxing/wiki/Barcode-Contents#wifi-network-config-android
        // Special characters `\`, `;`, `,` and `:` should be escaped with a backslash
        fn filter_credentials(&self, field: &str) -> String {
            // N.B. If performance problems ever crop up, this might be more performant
            // with regex replace_all 
            return field.to_string()
                .replace(r#"\"#, r#"\\"#)
                .replace(r#"""#, r#"\""#)
                .replace(r#";"#, r#"\;"#)
                .replace(r#":"#, r#"\:"#);
        }
        
        // the Encrption field in the Wifi QR code fails on iOS devices if it is
        // not provided in an uppercase format. Android devices are case insensitive,
        // so the encryption field is passed through as uppercase now.
        fn filter_encr(&self, field: &str) -> String {
            if field != "nopass" && !self.encr.is_empty() {
                return field.to_string().to_uppercase();
            }
            return field.to_string();
        }
       
        // Call the wifi_auth! macro to generate a qr-string and/or return any errors that 
        // need to be raised to the caller. Note: format does not enforce an encryption type, it is
        // up to the end user to use the right value if one is provided.
        pub fn format(&self) -> Result<String, &'static str> {
            // empty password ->
            //  * is password empty and ssid hidden? => set T:nopass and H:
            //  * is encryption type empty? => set nopass
            //  * hidden ssid? => add H:
            // plain format
            // unrecoverable errors:
            // * ssid has no password, but sets a T type
            // * sets a password, but sets T type to nopass

            if self.encr == "nopass" || self.encr.is_empty() {
                if !self.pass.is_empty() {
                    return Err("With nopass as the encryption type (or unset encryption type), the password field should be empty. 
                        (Encryption should probably be set to something like wpa2)")
                }
            }
    
            if self.pass.is_empty() || self.pass == "" {
                // Error condition: Password is empty, and the T (encr) type is not "nopass" / not empty
                if self.encr != "nopass" && !self.encr.is_empty() {
                    return Err("The encryption method requested requires a password.")
                }

                if self.encr.is_empty() || self.encr == "nopass" {
                    if self.hidden {
                        return Ok(format!(
                            wifi_auth!(nopass_hidden),
                            self.filter_credentials(&self.ssid),
                            &self.hidden,
                        ));
                    } else if self.pass == "" {
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
                ))
            } else {
                return Ok(format!(
                    wifi_auth!(),
                    self.filter_credentials(&self.filter_encr(&self.encr)),
                    self.filter_credentials(&self.ssid),
                    self.filter_credentials(&self.pass)
                ))
            }
       }

        // Transform the QR Wifi connection string into a Vec<char> for use with manual_encode()
        pub fn format_vec(&self) -> Vec<char> {
            return Credentials::format(&self).unwrap().chars().collect();
        }
    }

    // returns a new Credentials struct given Wifi credentials. This data is not validated,
    // nor formatted into a QR code string. Use .format() to do this
    pub fn auth(_ssid: Option<&str>, _password: Option<&str>, _encr: Option<&str>, _hidden: bool) -> Credentials {
        return self::Credentials::new(_ssid, _password, _encr, _hidden);
    }

    // generates a qrcode from a Credentials configuration 
    pub fn encode(config: &Credentials) -> Result<QrCode, DataTooLong> {
        let q = QrCode::encode_text(&config.format().unwrap(), QrCodeEcc::High)?;
        Ok(q)
    }

    // manual_encode isn't intended for use externally, but exists to compare between the
    // automated encoder and this manual_encode version
    // https://docs.rs/qrcodegen/latest/src/qrcodegen/lib.rs.html#151
    pub fn manual_encode(config: &Credentials, error_level: QrCodeEcc, lowest_version: qrcodegen::Version, 
        highest_version: qrcodegen::Version, mask_level: Option<Mask>) -> QrCode {

        let wifi: Vec<char> = config.format_vec();
        let segs: Vec<QrSegment> = QrSegment::make_segments(&wifi);

        return QrCode::encode_segments_advanced(
            &segs,
            error_level,
            lowest_version,
            highest_version,
            mask_level,
            true,
        ).unwrap();
    }

    pub fn make_svg(qrcode: &QrCode) -> String {
        return qrcode.to_svg_string(4);
    }

    // console_qr
    // generate a wifi qr code that can be output to the console for quick scanning
    // parameters:
    // - qrcode: encoded qrcode
    // return:
    // - this prints a block of text directly to the console
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

    // make_image
    // qrcode: Is an encoded qrcode
    // scale: The scaling factor to apply to the qrcode
    // border_size: How large to make the quiet zone
    // This returns an ImageBuffer<> that can be saved using save_image(), or passed on
    // for further manipulation by the caller
    pub fn make_image(qrcode: &QrCode, scale: i32, border_size: i32) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
        let new_qr_size = qrcode.size() * scale;

        // --- Initialize to a white canvas with the alpha layer pre-set ---
        let mut image = ImageBuffer::from_pixel(
            (new_qr_size + border_size * 2).try_into().unwrap(),
            (new_qr_size + border_size * 2).try_into().unwrap(),
            LumaA([255, 255]),
        );

        // --- Draw QR w/scale ---
        for y in 0..new_qr_size {
            for x in 0..new_qr_size {
                if qrcode.get_module(x, y) {
                    draw_filled_rect_mut(
                        &mut image,
                        Rect::at(
                            (x * scale) + border_size as i32,
                            (y * scale) + border_size as i32,
                        ).of_size(scale as u32, scale as u32),
                        LumaA([0, 255]),
                    );
                } else {
                    draw_filled_rect_mut(
                        &mut image,
                        Rect::at(
                            (x * scale) + border_size as i32,
                            (y * scale) + border_size as i32,
                        ).of_size(scale as u32, scale as u32),
                        LumaA([255, 255]),
                    );
                }
            }
        }

        return image;
    }

    // save_image
    // image: ImageBuffer<>
    // save_file: file to save the image into
    pub fn save_image(image: &ImageBuffer<LumaA<u8>, Vec<u8>>, save_file: String) {
        let _ = image.save(save_file).unwrap();
    }
}
