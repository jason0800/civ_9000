use macroquad::prelude::*;
use spade::{DelaunayTriangulation, Point2, Triangulation, HasPosition};

// 1. Define our survey point structure
#[derive(Clone, Copy, Debug)]
struct SurveyPoint {
    x: f64,
    y: f64,
    z: f64,
}

// Allow spade to understand our coordinates
impl HasPosition for SurveyPoint {
    type Scalar = f64;
    fn position(&self) -> Point2<f64> {
        Point2::new(self.x, self.y)
    }
}

#[macroquad::main("CIV 9000")]
async fn main() {
    // 2. Initialize the TIN math engine
    let mut triangulation: DelaunayTriangulation<SurveyPoint> = DelaunayTriangulation::new();
    
    // Sample site data (The "Mound")
    let points = vec![
        SurveyPoint { x: -2.0, y: -2.0, z: 0.0 },
        SurveyPoint { x: 2.0, y: -2.0, z: 0.0 },
        SurveyPoint { x: 2.0, y: 2.0, z: 0.0 },
        SurveyPoint { x: -2.0, y: 2.0, z: 0.0 },
        SurveyPoint { x: 0.0, y: 0.0, z: 2.5 }, 
    ];

    for pt in points {
        triangulation.insert(pt).expect("Failed to insert point");
    }

    // --- CAMERA STATE ---
    let mut longitude: f32 = 0.8; 
    let mut latitude: f32 = 0.5;  
    let mut zoom: f32 = 10.0;

    loop {
        // --- INPUT: Orbit ---
        if is_mouse_button_down(MouseButton::Left) {
            let delta = mouse_delta_position();
            longitude += delta.x * 3.0; 
            latitude -= delta.y * 3.0; // Inverted: drag down to tilt up
        }

        // Clamp latitude to prevent the camera from flipping over the top/bottom
        latitude = latitude.clamp(-1.5, 1.5);

        // --- INPUT: Normalized Zoom ---
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            // signum() makes every scroll click weigh the same (1.0 or -1.0)
            let direction = wheel.signum(); 
            // Percentage-based zoom (10% per click) prevents skipping the middle ground
            zoom -= direction * (zoom * 0.1); 
        }
        zoom = zoom.clamp(0.5, 50.0);

        clear_background(Color::new(0.1, 0.1, 0.12, 1.0));

        // --- MATH: Calculate Camera Position (Turntable Orbit) ---
        let x = zoom * longitude.sin() * latitude.cos();
        let y = zoom * latitude.sin(); 
        let z = zoom * longitude.cos() * latitude.cos();

        set_camera(&Camera3D {
            position: vec3(x, y, z),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            ..Default::default() 
        });

        // --- RENDER: Draw the TIN ---
        for face in triangulation.inner_faces() {
            let v = face.vertices();
            
            // Map our coordinates (Civil Z = Macroquad Y)
            let p1 = vec3(v[0].data().x as f32, v[0].data().z as f32, v[0].data().y as f32);
            let p2 = vec3(v[1].data().x as f32, v[1].data().z as f32, v[1].data().y as f32);
            let p3 = vec3(v[2].data().x as f32, v[2].data().z as f32, v[2].data().y as f32);

            // Draw the Wireframe Edges
            draw_line_3d(p1, p2, WHITE);
            draw_line_3d(p2, p3, WHITE);
            draw_line_3d(p3, p1, WHITE);

            // Draw the Vertices
            draw_sphere(p1, 0.05, None, GREEN);
        }

        // Back to 2D for text/UI
        set_default_camera();
        
        draw_rectangle(10.0, 10.0, 350.0, 30.0, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_text(&format!("CIV 9000 | Zoom: {:.2}", zoom), 20.0, 30.0, 20.0, WHITE);
        
        next_frame().await
    }
}