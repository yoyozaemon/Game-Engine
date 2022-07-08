#![allow(non_snake_case)]

#[cfg(test)]
mod texture_tests
{
    // Renderer
    use crate::renderer::texture::Image;

    #[test]
    fn ImageFormatsTest()
    {
        std::env::set_current_dir("../").expect("Failed setting working dir");

        // PNG
        Image::LoadFromFile("rusty_engine/assets/test_assets/png_test_image.png", false);
        Image::LoadFromFile("rusty_engine/assets/test_assets/png_test_image.png", true);

        // JPG
        Image::LoadFromFile("rusty_engine/assets/test_assets/jpg_test_image.jpg", false);
        Image::LoadFromFile("rusty_engine/assets/test_assets/jpg_test_image.jpg", true);

        // BMP
        Image::LoadFromFile("rusty_engine/assets/test_assets/bmp_test_image.bmp", false);
        Image::LoadFromFile("rusty_engine/assets/test_assets/bmp_test_image.bmp", true);

        // TGA
        Image::LoadFromFile("rusty_engine/assets/test_assets/tga_test_image.tga", false);
        Image::LoadFromFile("rusty_engine/assets/test_assets/tga_test_image.tga", true);

        // HDR
        Image::LoadFromFile("rusty_engine/assets/test_assets/hdr_test_image.hdr", false);
        Image::LoadFromFile("rusty_engine/assets/test_assets/hdr_test_image.hdr", true);
    }
}