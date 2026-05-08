use macroquad::prelude::*;
use macroquad::ui::root_ui; // Import UI
use spade::{DelaunayTriangulation, Point2, Triangulation, HasPosition};

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
    let mut triangulation: DelaunayTriangulation<SurveyPoint> = DelaunayTriangulation::new();
    
    let mut points = Vec::new();
    let grid_size = 10;
    let spacing = 1.0;

    for i in 0..grid_size {
        for j in 0..grid_size {
            let x = i as f64 * spacing - (grid_size as f64 / 2.0);
            let y = j as f64 * spacing - (grid_size as f64 / 2.0);
            
            // This math creates a natural "swale" or rolling hill effect
            // Z = cos(x/2) + sin(y/2) + a small random jitter for realism
            let z = (x * 0.4).cos() + (y * 0.4).sin() + (i as f64 * 0.05);
            
            points.push(SurveyPoint { x, y, z });
        }
    }

    for pt in points {
        triangulation.insert(pt).expect("Failed to insert point");
    }

    // --- STATE ---
    let mut longitude: f32 = 0.8; 
    let mut latitude: f32 = 0.5;  
    let mut zoom: f32 = 10.0;
    let mut target = vec3(0.0, 0.0, 0.0);
    let mut show_points = true; // Toggle variable

    loop {
        let delta = mouse_delta_position();

        // Only allow orbit/pan if we aren't clicking on a UI element
        // (Prevents the camera from spinning when you click the button)
        if !root_ui().is_mouse_over(mouse_position().into()) {
            if is_mouse_button_down(MouseButton::Left) {
                longitude += delta.x * 3.0; 
                latitude -= delta.y * 3.0; 
            }

            if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
                let look_dir = vec3(longitude.sin() * latitude.cos(), latitude.sin(), longitude.cos() * latitude.cos()).normalize();
                let right = look_dir.cross(vec3(0.0, 1.0, 0.0)).normalize();
                let up_vec = right.cross(look_dir).normalize();
                target -= right * delta.x * zoom * 0.8;
                target -= up_vec * delta.y * zoom * 0.8;
            }
        }

        latitude = latitude.clamp(-1.5, 1.5);

        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            zoom -= wheel.signum() * (zoom * 0.1); 
        }
        zoom = zoom.clamp(0.1, 100.0);

        clear_background(Color::new(0.1, 0.1, 0.12, 1.0));

        let cam_x = target.x + zoom * longitude.sin() * latitude.cos();
        let cam_y = target.y + zoom * latitude.sin(); 
        let cam_z = target.z + zoom * longitude.cos() * latitude.cos();

        set_camera(&Camera3D {
            position: vec3(cam_x, cam_y, cam_z),
            up: vec3(0.0, 1.0, 0.0),
            target: target,
            ..Default::default() 
        });

        // --- RENDER: Edges ---
        for face in triangulation.inner_faces() {
            let v = face.vertices();
            let p1 = vec3(v[0].data().x as f32, v[0].data().z as f32, v[0].data().y as f32);
            let p2 = vec3(v[1].data().x as f32, v[1].data().z as f32, v[1].data().y as f32);
            let p3 = vec3(v[2].data().x as f32, v[2].data().z as f32, v[2].data().y as f32);
            draw_line_3d(p1, p2, WHITE);
            draw_line_3d(p2, p3, WHITE);
            draw_line_3d(p3, p1, WHITE);
        }

        // --- RENDER: Points (Conditional) ---
        if show_points {
            for vertex in triangulation.vertices() {
                let data = vertex.data();
                let pos = vec3(data.x as f32, data.z as f32, data.y as f32);
                draw_sphere(pos, 0.05, None, GREEN);
            }
        }

        set_default_camera();
        
        // --- UI ---
        draw_rectangle(10.0, 10.0, 420.0, 30.0, Color::new(0.0, 0.0, 0.0, 0.5));
        draw_text(
            &format!("CIV 9000 | Zoom: {:.2}", zoom), 
            20.0, 30.0, 20.0, WHITE
        );

        // Add the Button
        let btn_label = if show_points { "Hide Points" } else { "Show Points" };
        if root_ui().button(vec2(10.0, 50.0), btn_label) {
            show_points = !show_points;
        }
        
        next_frame().await
    }
}