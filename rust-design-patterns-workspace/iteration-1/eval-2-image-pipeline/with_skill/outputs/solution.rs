use std::fmt;

// ─── Core Image type ───────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Image {
    fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self { width, height, pixels }
    }
}

// ─── Filter trait (Strategy pattern with dynamic dispatch) ─────────

/// Every image filter implements this trait. Adding a new filter means
/// creating a new struct + impl — no existing code needs to change.
trait ImageFilter: fmt::Debug {
    /// Apply the filter to the image, returning Ok or an error description.
    fn apply(&self, image: &mut Image) -> Result<(), FilterError>;

    /// Human-readable name for logging/debugging.
    fn name(&self) -> &str;
}

// ─── Error handling ────────────────────────────────────────────────

#[derive(Debug)]
struct FilterError {
    filter_name: String,
    message: String,
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Filter '{}' failed: {}", self.filter_name, self.message)
    }
}

// ─── Concrete filters ─────────────────────────────────────────────

#[derive(Debug)]
struct Resize {
    scale: f64,
}

impl Resize {
    fn new(scale: f64) -> Self {
        Self { scale }
    }
}

impl ImageFilter for Resize {
    fn apply(&self, image: &mut Image) -> Result<(), FilterError> {
        if self.scale <= 0.0 {
            return Err(FilterError {
                filter_name: self.name().to_string(),
                message: format!("scale must be positive, got {}", self.scale),
            });
        }
        image.width = (image.width as f64 * self.scale) as u32;
        image.height = (image.height as f64 * self.scale) as u32;
        // Actual pixel resampling would go here.
        println!("  Resized to {}x{}", image.width, image.height);
        Ok(())
    }

    fn name(&self) -> &str {
        "resize"
    }
}

#[derive(Debug)]
struct Blur {
    radius: f64,
}

impl Blur {
    fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl ImageFilter for Blur {
    fn apply(&self, image: &mut Image) -> Result<(), FilterError> {
        if self.radius < 0.0 {
            return Err(FilterError {
                filter_name: self.name().to_string(),
                message: format!("radius must be non-negative, got {}", self.radius),
            });
        }
        // Gaussian blur logic would go here.
        println!("  Applied blur with radius {}", self.radius);
        Ok(())
    }

    fn name(&self) -> &str {
        "blur"
    }
}

#[derive(Debug)]
struct Sharpen {
    amount: f64,
}

impl Sharpen {
    fn new(amount: f64) -> Self {
        Self { amount }
    }
}

impl ImageFilter for Sharpen {
    fn apply(&self, image: &mut Image) -> Result<(), FilterError> {
        // Unsharp mask logic would go here.
        println!("  Applied sharpen with amount {}", self.amount);
        Ok(())
    }

    fn name(&self) -> &str {
        "sharpen"
    }
}

#[derive(Debug)]
struct Grayscale;

impl ImageFilter for Grayscale {
    fn apply(&self, image: &mut Image) -> Result<(), FilterError> {
        // Convert RGB to luminance, etc.
        println!("  Converted to grayscale");
        Ok(())
    }

    fn name(&self) -> &str {
        "grayscale"
    }
}

#[derive(Debug)]
struct Rotate {
    degrees: f64,
}

impl Rotate {
    fn new(degrees: f64) -> Self {
        Self { degrees }
    }
}

impl ImageFilter for Rotate {
    fn apply(&self, image: &mut Image) -> Result<(), FilterError> {
        // Rotation logic; swap dimensions for 90/270.
        if (self.degrees % 90.0).abs() < f64::EPSILON
            && ((self.degrees / 90.0) as i32 % 2 != 0)
        {
            std::mem::swap(&mut image.width, &mut image.height);
        }
        println!("  Rotated by {} degrees", self.degrees);
        Ok(())
    }

    fn name(&self) -> &str {
        "rotate"
    }
}

// ─── Pipeline (Chain of Responsibility as Vec) ─────────────────────

/// A pipeline owns an ordered sequence of filters and applies them
/// to an image. Filters are stored as trait objects because the
/// pipeline is heterogeneous — it mixes different concrete filter types.
#[derive(Debug)]
struct Pipeline {
    filters: Vec<Box<dyn ImageFilter>>,
}

impl Pipeline {
    fn new() -> Self {
        Self { filters: Vec::new() }
    }

    /// Add a filter to the end of the pipeline.
    fn add_filter(mut self, filter: Box<dyn ImageFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Run every filter in order. Stops on the first error.
    fn execute(&self, image: &mut Image) -> Result<(), FilterError> {
        println!("Running pipeline ({} filters):", self.filters.len());
        for filter in &self.filters {
            println!("  [{}]", filter.name());
            filter.apply(image)?;
        }
        println!("Pipeline complete. Final size: {}x{}", image.width, image.height);
        Ok(())
    }
}

// ─── Example usage ─────────────────────────────────────────────────

fn main() {
    let mut img = Image::new(1920, 1080, vec![0u8; 1920 * 1080 * 4]);

    // Build the pipeline — all filter names are checked at compile time.
    // A typo like `Blurr` would be a compile error, not a runtime panic.
    let pipeline = Pipeline::new()
        .add_filter(Box::new(Resize::new(0.5)))
        .add_filter(Box::new(Blur::new(2.0)))
        .add_filter(Box::new(Sharpen::new(1.5)))
        .add_filter(Box::new(Grayscale))
        .add_filter(Box::new(Rotate::new(90.0)));

    match pipeline.execute(&mut img) {
        Ok(()) => println!("All filters applied successfully."),
        Err(e) => eprintln!("Error: {e}"),
    }
}

// ─── Adding a new filter (what a team member does) ─────────────────
//
// 1. Define your struct with its parameters:
//      #[derive(Debug)]
//      struct Sepia { intensity: f64 }
//
// 2. Implement ImageFilter:
//      impl ImageFilter for Sepia {
//          fn apply(&self, image: &mut Image) -> Result<(), FilterError> { ... }
//          fn name(&self) -> &str { "sepia" }
//      }
//
// 3. Use it:
//      pipeline.add_filter(Box::new(Sepia { intensity: 0.8 }))
//
// No match arms to update. No central registry. No string-based dispatch.
