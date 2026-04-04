// ---------------------------------------------------------------------------
// Image Pipeline — Command Pattern + Trait Objects
// ---------------------------------------------------------------------------
//
// Each filter is a standalone struct that implements the `Filter` trait.
// Adding a new filter means adding a new file/struct — no existing code changes.
// The pipeline is built at compile time so typos are caught by the compiler.
// ---------------------------------------------------------------------------

/// Raw image data.
#[derive(Debug, Clone)]
struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Core trait
// ---------------------------------------------------------------------------

/// Every image filter implements this trait.
trait Filter {
    /// A human-readable name, useful for logging / debugging.
    fn name(&self) -> &str;

    /// Apply the filter to `image` in place.
    fn apply(&self, image: &mut Image);
}

// ---------------------------------------------------------------------------
// Concrete filters
// ---------------------------------------------------------------------------

struct Resize {
    scale: f64,
}

impl Resize {
    fn new(scale: f64) -> Self {
        Self { scale }
    }
}

impl Filter for Resize {
    fn name(&self) -> &str {
        "resize"
    }

    fn apply(&self, image: &mut Image) {
        image.width = (image.width as f64 * self.scale) as u32;
        image.height = (image.height as f64 * self.scale) as u32;
        // actual resize logic would go here
    }
}

// ---

struct Blur {
    radius: f64,
}

impl Blur {
    fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl Filter for Blur {
    fn name(&self) -> &str {
        "blur"
    }

    fn apply(&self, image: &mut Image) {
        let _radius = self.radius;
        // blur logic would go here
    }
}

// ---

struct Sharpen {
    amount: f64,
}

impl Sharpen {
    fn new(amount: f64) -> Self {
        Self { amount }
    }
}

impl Filter for Sharpen {
    fn name(&self) -> &str {
        "sharpen"
    }

    fn apply(&self, image: &mut Image) {
        let _amount = self.amount;
        // sharpen logic would go here
    }
}

// ---

struct Grayscale;

impl Filter for Grayscale {
    fn name(&self) -> &str {
        "grayscale"
    }

    fn apply(&self, image: &mut Image) {
        // grayscale conversion logic would go here
        let _ = &image.pixels;
    }
}

// ---

struct Rotate {
    degrees: f64,
}

impl Rotate {
    fn new(degrees: f64) -> Self {
        Self { degrees }
    }
}

impl Filter for Rotate {
    fn name(&self) -> &str {
        "rotate"
    }

    fn apply(&self, image: &mut Image) {
        let _degrees = self.degrees;
        // rotation logic would go here
    }
}

// ---------------------------------------------------------------------------
// Pipeline (builder pattern for ergonomic chaining)
// ---------------------------------------------------------------------------

struct Pipeline {
    filters: Vec<Box<dyn Filter>>,
}

impl Pipeline {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Append any filter to the pipeline. Returns `self` so calls can be chained.
    fn add(mut self, filter: impl Filter + 'static) -> Self {
        self.filters.push(Box::new(filter));
        self
    }

    /// Execute every filter in order against `image`.
    fn execute(&self, image: &mut Image) {
        for filter in &self.filters {
            println!("Applying filter: {}", filter.name());
            filter.apply(image);
        }
    }
}

// ---------------------------------------------------------------------------
// Usage example
// ---------------------------------------------------------------------------

fn main() {
    let mut img = Image {
        width: 1920,
        height: 1080,
        pixels: vec![0u8; 1920 * 1080 * 4],
    };

    // Build the pipeline — every filter name is checked at compile time.
    // A typo like `Blurr` simply won't compile.
    let pipeline = Pipeline::new()
        .add(Resize::new(0.5))
        .add(Blur::new(2.0))
        .add(Sharpen::new(1.5))
        .add(Grayscale)
        .add(Rotate::new(90.0));

    pipeline.execute(&mut img);

    println!(
        "Final image size: {}x{} ({} bytes)",
        img.width,
        img.height,
        img.pixels.len()
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_halves_dimensions() {
        let mut img = Image {
            width: 100,
            height: 200,
            pixels: vec![],
        };
        Resize::new(0.5).apply(&mut img);
        assert_eq!(img.width, 50);
        assert_eq!(img.height, 100);
    }

    #[test]
    fn pipeline_applies_filters_in_order() {
        let mut img = Image {
            width: 100,
            height: 100,
            pixels: vec![],
        };

        let pipeline = Pipeline::new()
            .add(Resize::new(2.0))   // 100 -> 200
            .add(Resize::new(0.5));  // 200 -> 100

        pipeline.execute(&mut img);
        assert_eq!(img.width, 100);
        assert_eq!(img.height, 100);
    }

    #[test]
    fn empty_pipeline_is_no_op() {
        let mut img = Image {
            width: 42,
            height: 42,
            pixels: vec![],
        };
        Pipeline::new().execute(&mut img);
        assert_eq!(img.width, 42);
    }

    /// Demonstrates that adding a custom filter requires zero changes to Pipeline.
    #[test]
    fn custom_third_party_filter() {
        struct Invert;
        impl Filter for Invert {
            fn name(&self) -> &str { "invert" }
            fn apply(&self, image: &mut Image) {
                for px in image.pixels.iter_mut() {
                    *px = 255 - *px;
                }
            }
        }

        let mut img = Image {
            width: 1,
            height: 1,
            pixels: vec![0, 100, 200, 255],
        };
        Pipeline::new().add(Invert).execute(&mut img);
        assert_eq!(img.pixels, vec![255, 155, 55, 0]);
    }
}
