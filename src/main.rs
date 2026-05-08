use macroquad::prelude::*;
use spade::{DelaunayTriangulation, Point2, Triangulation, HasPosition};

// 1. Define our survey point structure
#[derive(Clone, Copy, Debug)]
struct SurveyPoint {
    x: f64,
    y: f64,
    z: f64,
}

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
    let mut target = vec3(0.0, 0.0, 0.0); // The point we are looking at

    loop {
        let delta = mouse_delta_position();

        // --- INPUT: Orbit (Left Click) ---
        if is_mouse_button_down(MouseButton::Left) {
            longitude += delta.x * 3.0; 
            latitude -= delta.y * 3.0; 
        }

        // --- INPUT: Pan (Right Click or Middle Mouse) ---
        if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
            // Calculate relative vectors so panning follows the camera orientation
            let look_dir = vec3(longitude.sin() * latitude.cos(), latitude.sin(), longitude.cos() * latitude.cos()).normalize();
            let right = look_dir.cross(vec3(0.0, 1.0, 0.0)).normalize();
            let up_vec = right.cross(look_dir).normalize();

            // Pan speed scales with zoom so it feels consistent
            target -= right * delta.x * zoom * 0.8;
            target -= up_vec * delta.y * zoom * 0.8;
        }

        latitude = latitude.clamp(-1.5, 1.5);

        // --- INPUT: Zoom ---
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            zoom -= wheel.signum() * (zoom * 0.1); 
        }
        zoom = zoom.clamp(0.1, 100.0);

        clear_background(Color::new(0.1, 0.1, 0.12, 1.0));

        // --- MATH: Calculate Camera Position ---
        let cam_x = target.x + zoom * longitude.sin() * latitude.cos();
        let cam_y = target.y + zoom * latitude.sin(); 
        let cam_z = target.z + zoom * longitude.cos() * latitude.cos();

        set_camera(&Camera3D {
            position: vec3(cam_x, cam_y, cam_z),
            up: vec3(0.0, 1.0, 0.0),
            target: target,
            ..Default::default() 
        });

        // --- RENDER: Draw the TIN ---
        for face in triangulation.inner_faces() {
            let v = face.vertices();
            
            // Map our coordinates (Civil Z = Macroquad Y)
            let p1 = vec3(v[0].data().x as f32, v[0].data().z as f32, v[0].data().y as f32);
            let p2 = vec3(v[1].data().x as f32, v[1].data().z as f32, v[1].data().y as f32);
            let p3 = vec3(v[2].data().x as f32, v[2].data().z as f32, v[2].data().y as f32);

            draw_line_3d(p1, p2, WHITE);
            draw_line_3d(p2, p3, WHITE);
            draw_line_3d(p3, p1, WHITE);
            draw_sphere(p1, 0.05, None, GREEN);
        }

        // Return to 2D for UI
        set_default_camera();
        
        draw_rectangle(10.0, 10.0, 420.0, 30.0, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_text(
            &format!("CIV 9000 | Orbit: Left | Pan: Right | Zoom: {:.2}", zoom), 
            20.0, 30.0, 20.0, WHITE
        );
        
        next_frame().await
    }
}