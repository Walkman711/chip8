As an AI language model, I am not capable of creating a canvas for you. However, here's an example Rust code that draws a 2D array on a canvas using the `web-sys` crate:

```rust
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn draw_array_on_canvas(mut ctx: CanvasRenderingContext2d, array: [[u8; 4]; 4]) {
    // Clear the canvas
    ctx.clear_rect(0.0, 0.0, 400.0, 400.0);

    // Draw the array
    let cell_size = 100.0;

    for i in 0..4 {
        for j in 0..4 {
            let x = j as f64 * cell_size;
            let y = i as f64 * cell_size;

            match array[i][j] {
                0 => {
                    // Draw an empty cell
                    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
                    ctx.fill_rect(x, y, cell_size, cell_size);
                }
                1 => {
                    // Draw a filled cell
                    ctx.set_fill_style(&JsValue::from_str("#000000"));
                    ctx.fill_rect(x, y, cell_size, cell_size);
                }
                _ => {}
            }
        }
    }
}

fn main() {
    // Get the canvas element
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("my-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    // Get the canvas context
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    // Define a test array
    let test_array = [[1, 0, 1, 0], [0, 1, 0, 1], [1, 0, 1, 0], [0, 1, 0, 1]];

    // Draw the array on the canvas
    draw_array_on_canvas(ctx, test_array);
}
```

In this example, the `draw_array_on_canvas` function takes a `CanvasRenderingContext2d` object and a 2D array of `u8` values. It loops through the array and draws rectangles on the canvas based on the values in the array. The `main` function gets the canvas element, retrieves the canvas context, defines a test array, and calls `draw_array_on_canvas` to draw the array on the canvas.
