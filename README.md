# TTGO T-Display Image Slideshow

This project contains an expansion to the code in
[esp32-image-display](https://github.com/lily-mara/esp32-image-display), which
only supported displaying a single image. This project will scan every `.bmp`
file in the `images` directory and allow you to loop through them by using the
buttons on the face of the T-Display.

## Usage

You must create BMP images that are 135x240 and place them in the `images`
directory with a lowercase `.bmp` extension. You can then use `cargo run` to
compile and upload the program to your T-Display.
