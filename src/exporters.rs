extern crate qrcodegen;

pub mod methods {
    use qrcodegen::QrCode;

    use std::convert::TryInto;

    use image::{ImageBuffer, LumaA};
    use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};

    /// returns an ImageBuffer<> that can be saved using save_image(), or passed on
    /// for further manipulation by the caller
    ///
    ///
    /// * qrcode: Is an encoded qrcode
    ///
    /// * scale: The scaling factor to apply to the qrcode
    ///
    /// * border_size: How large to make the quiet zone
    pub fn make_image(
        qrcode: &QrCode,
        scale: i32,
        border_size: i32,
    ) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
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
                        )
                        .of_size(scale as u32, scale as u32),
                        LumaA([0, 255]),
                    );
                } else {
                    draw_filled_rect_mut(
                        &mut image,
                        Rect::at(
                            (x * scale) + border_size as i32,
                            (y * scale) + border_size as i32,
                        )
                        .of_size(scale as u32, scale as u32),
                        LumaA([255, 255]),
                    );
                }
            }
        }

        image
    }

    /// saves an image to a file
    ///
    /// * image: ImageBuffer<>
    ///
    /// * save_file: file path to save the image into. ImageBuffer only supports jpeg and png extensions.
    pub fn save_image(
        image: &ImageBuffer<LumaA<u8>, Vec<u8>>,
        save_file: String,
    ) -> Result<(), image::ImageError> {
        match image.save(save_file) {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// returns a QR code that can be interpreted by an SVG reader
    ///
    /// * qr: &QrCode
    ///
    /// * border: size of border to apply to the SVG
    pub fn to_svg_string(qr: &QrCode, border: i32) -> String {
        /* to_svg_string is derived from Project Nayuki's QR Code Generator
         *
         * Copyright (c) Project Nayuki. (MIT License)
         * https://www.nayuki.io/page/qr-code-generator-library
         *
         * Permission is hereby granted, free of charge, to any person obtaining a copy of
         * this software and associated documentation files (the "Software"), to deal in
         * the Software without restriction, including without limitation the rights to
         * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
         * the Software, and to permit persons to whom the Software is furnished to do so,
         * subject to the following conditions:
         * - The above copyright notice and this permission notice shall be included in
         *   all copies or substantial portions of the Software.
         * - The Software is provided "as is", without warranty of any kind, express or
         *   implied, including but not limited to the warranties of merchantability,
         *   fitness for a particular purpose and noninfringement. In no event shall the
         *   authors or copyright holders be liable for any claim, damages or other
         *   liability, whether in an action of contract, tort or otherwise, arising from,
         *   out of or in connection with the Software or the use or other dealings in the
         *   Software.
         *
         */

        assert!(border >= 0, "Border must be non-negative");
        let mut result = String::new();
        result += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
        result += "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n";
        let dimension = qr
            .size()
            .checked_add(border.checked_mul(2).unwrap())
            .unwrap();
        result += &format!(
	        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {0} {0}\" stroke=\"none\">\n", dimension);
        result += "\t<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n";
        result += "\t<path d=\"";
        for y in 0..qr.size() {
            for x in 0..qr.size() {
                if qr.get_module(x, y) {
                    if x != 0 || y != 0 {
                        result += " ";
                    }
                    result += &format!("M{},{}h1v1h-1z", x + border, y + border);
                }
            }
        }
        result += "\" fill=\"#000000\"/>\n";
        result += "</svg>\n";
        result
    }
}
