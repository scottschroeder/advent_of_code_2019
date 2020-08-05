use self::space_image_format::Image;
use crate::util::parse_digits;
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let data = parse_digits(input)?;
    let img = Image::new(data, 25, 6);

    let mut min_layer = None;
    for layer in img.layers() {
        let cs = checksum_layer(layer.as_ref());
        if cs.0 <= min_layer.get_or_insert(cs).0 {
            min_layer = Some(cs)
        }
    }
    let img_cs = min_layer
        .map(|(_, c1, c2)| c1 * c2)
        .expect("no layers in image");

    Ok(format!("{}", img_cs))
}

pub fn part2(input: &str) -> Result<String> {
    let data = parse_digits(input)?;
    let img = Image::new(data, 25, 6);
    let canvas = img.render();
    Ok(format!("{}", canvas))
}

fn checksum_layer(data: &[u8]) -> (u64, u64, u64) {
    let mut c0 = 0;
    let mut c1 = 0;
    let mut c2 = 0;
    for pixel in data {
        match pixel {
            0 => c0 += 1,
            1 => c1 += 1,
            2 => c2 += 1,
            _ => {}
        }
    }
    (c0, c1, c2)
}

mod space_image_format {
    use std::fmt;

    pub struct Layers<'a> {
        inner: &'a Image,
        layer: usize,
    }

    impl<'a> Layers<'a> {
        fn new(inner: &'a Image) -> Layers<'a> {
            Layers { inner, layer: 0 }
        }
    }

    impl<'a> Iterator for Layers<'a> {
        type Item = Layer<'a>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.layer < self.inner.depth {
                let view = self.inner.layer(self.layer);
                self.layer += 1;
                Some(view)
            } else {
                None
            }
        }
    }

    pub struct Canvas {
        inner: Vec<u8>,
        width: usize,
        height: usize,
    }

    impl Canvas {
        fn new(width: usize, height: usize) -> Canvas {
            Canvas {
                inner: vec![2; width * height],
                width,
                height,
            }
        }

        fn add_lower_layer(&mut self, layer: &Layer) {
            assert_eq!(layer.width, self.width);
            assert_eq!(layer.height, self.height);
            assert_eq!(layer.inner.len(), self.inner.len());
            for (c, l) in self.inner.iter_mut().zip(layer.inner.iter()) {
                log::trace!("c={}, l={}", c, l);
                if *c >= 2 {
                    *c = *l;
                }
            }
        }
    }

    impl fmt::Debug for Canvas {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for row_idx in 0..self.height {
                for col_idx in 0..self.width {
                    let pix = self.inner[self.width * row_idx + col_idx];
                    write!(f, "{}", pix)?;
                }
                write!(f, "\n")?;
            }
            Ok(())
        }
    }
    impl fmt::Display for Canvas {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for row_idx in 0..self.height {
                if row_idx != 0 {
                    write!(f, "\n")?;
                }
                for col_idx in 0..self.width {
                    let pix = self.inner[self.width * row_idx + col_idx];
                    match pix {
                        1 => write!(f, "#")?,
                        _ => write!(f, " ")?,
                    }
                }
            }
            Ok(())
        }
    }

    pub struct Layer<'a> {
        inner: &'a [u8],
        width: usize,
        height: usize,
    }

    impl<'a> AsRef<[u8]> for Layer<'a> {
        fn as_ref(&self) -> &[u8] {
            self.inner
        }
    }

    pub struct Image {
        data: Vec<u8>,
        width: usize,
        height: usize,
        depth: usize,
        layer_size: usize,
    }

    impl Image {
        pub fn new(data: Vec<u8>, width: usize, height: usize) -> Image {
            let layer_size = width * height;
            let depth = data.len() / layer_size;

            Image {
                data,
                width,
                height,
                depth,
                layer_size,
            }
        }

        pub fn layer(&self, idx: usize) -> Layer {
            let start = idx * self.layer_size;
            let end = start + self.layer_size;
            let layer_view = &self.data[start..end];
            Layer {
                inner: layer_view,
                width: self.width,
                height: self.height,
            }
        }

        pub fn render(&self) -> Canvas {
            let mut c = Canvas::new(self.width, self.height);
            for l in self.layers() {
                c.add_lower_layer(&l)
            }
            c
        }

        pub fn layers(&self) -> Layers {
            Layers::new(self)
        }
    }

    impl fmt::Debug for Image {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut row_idx = 0;
            let mut col_idx = 0;
            let mut lay_idx = 0;

            write!(f, "Image {{")?;
            for pix in &self.data {
                if row_idx == 0 && col_idx == 0 {
                    write!(f, "\n\t Layer: {}", lay_idx)?;
                }
                if col_idx == 0 {
                    write!(f, "\n\t\t")?;
                }
                write!(f, "{}", pix)?;
                col_idx += 1;
                if col_idx >= self.width {
                    col_idx = 0;
                    row_idx += 1;
                }
                if row_idx >= self.height {
                    row_idx = 0;
                    lay_idx += 1;
                }
            }
            write!(f, "\n}}")?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn load_simple_image() {
            let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2];
            let img = Image::new(data, 3, 2);
            let mut layers = img.layers();
            assert_eq!(layers.next().unwrap().as_ref(), &[1, 2, 3, 4, 5, 6]);
            assert_eq!(layers.next().unwrap().as_ref(), &[7, 8, 9, 0, 1, 2]);
        }

        #[test]
        fn render_canvas_simple() {
            let data = vec![0, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 2, 0, 0, 0, 0];
            let img = Image::new(data, 2, 2);
            let canvas = img.render();
            assert_eq!(canvas.inner, vec![0, 1, 1, 0])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn parse_day8() {
        assert!(parse_digits(DAY8_INPUT).unwrap().len() > 0)
    }

    #[test]
    fn day8part1() {
        assert_eq!(part1(DAY8_INPUT).unwrap().as_str(), "2684")
    }

    #[test]
    fn day8part2() {
        assert_eq!(
            part2(DAY8_INPUT).unwrap().as_str().trim(),
            DAY8_PART2_OUTPUT.trim()
        )
    }
}
