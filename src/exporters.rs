extern crate qrcodegen;

pub mod svg {
    use qrcodegen::QrCode;

    pub fn to_svg_string(qr: &QrCode, border: i32) -> String {
        /* to_svg_string() is derived from Project Nayuki's QR Code Generator
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
