/// Calculate the sum of two numbers.
/// 
/// This function takes two integers as parameters and returns their sum.
/// It's a simple arithmetic operation that demonstrates proper documentation.
/// 
/// # Arguments
/// 
/// * `a` - The first number
/// * `b` - The second number
/// 
/// # Returns
/// 
/// The sum of `a` and `b`
/// 
/// # Examples
/// 
/// ```
/// let result = add(5, 3);
/// assert_eq!(result, 8);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Represents a point in 2D space.
/// 
/// This struct holds the coordinates of a point in a two-dimensional
/// coordinate system.
struct Point {
    /// The x-coordinate
    x: f64,
    /// The y-coordinate  
    y: f64,
}

/// Implementation of Point methods.
impl Point {
    /// Create a new point at the origin.
    fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    /// Create a new point with the given coordinates.
    fn new_with_coords(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Represents different shapes.
enum Shape {
    /// A circle with a radius
    Circle(f64),
    /// A rectangle with width and height
    Rectangle { width: f64, height: f64 },
}

/// Calculate the area of a shape.
fn calculate_area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(radius) => std::f64::consts::PI * radius * radius,
        Shape::Rectangle { width, height } => width * height,
    }
}