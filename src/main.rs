use macroquad::prelude::*;
use macroquad::ui::root_ui;
use macroquad::hash;
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

#[derive(PartialEq, Debug, Clone, Copy)]
enum ViewMode {
    Edit2D,
    View3D,
}

#[macroquad::main("CIV 9000")]
async fn main() {
    let mut triangulation: DelaunayTriangulation<SurveyPoint> = DelaunayTriangulation::new();
    let mut mode = ViewMode::Edit2D;
    let mut next_elevation = 0.0;
    let mut elevation_input = String::from("0.0");
    let mut pan2d = vec2(0.0, 0.0);
    let mut last_mouse_pos = vec2(0.0, 0.0);

    let mut longitude: f32 = 0.8; 
    let mut latitude: f32 = 0.5;  
    let mut zoom: f32 = 10.0;
    let mut target = vec3(0.0, 0.0, 0.0);

    loop {
        let (m_x, m_y) = mouse_position();
        let current_mouse_pos = vec2(m_x, m_y);
        let mouse_delta = current_mouse_pos - last_mouse_pos;
        last_mouse_pos = current_mouse_pos;

        if is_key_pressed(KeyCode::Tab) {
            mode = if mode == ViewMode::Edit2D { ViewMode::View3D } else { ViewMode::Edit2D };
        }

        clear_background(BLACK);

        match mode {
            ViewMode::Edit2D => {
                set_default_camera();

                if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
                    pan2d += mouse_delta;
                }

                if is_mouse_button_pressed(MouseButton::Left) && !root_ui().is_mouse_over(vec2(m_x, m_y)) {
                    let new_pt = SurveyPoint {
                        x: (m_x as f64 - screen_width() as f64 / 2.0 - pan2d.x as f64) / 50.0,
                        y: (m_y as f64 - screen_height() as f64 / 2.0 - pan2d.y as f64) / 50.0,
                        z: next_elevation as f64,
                    };
                    let _ = triangulation.insert(new_pt);
                }

                // 2D Wireframe
                for edge in triangulation.undirected_edges() {
                    let [v1, v2] = edge.vertices();
                    let p1 = vec2(v1.data().x as f32 * 50.0 + screen_width()/2.0 + pan2d.x, v1.data().y as f32 * 50.0 + screen_height()/2.0 + pan2d.y);
                    let p2 = vec2(v2.data().x as f32 * 50.0 + screen_width()/2.0 + pan2d.x, v2.data().y as f32 * 50.0 + screen_height()/2.0 + pan2d.y);
                    draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, BLUE);
                }

                // 2D Dots
                for vertex in triangulation.vertices() {
                    let data = vertex.data();
                    draw_circle(data.x as f32 * 50.0 + screen_width()/2.0 + pan2d.x, data.y as f32 * 50.0 + screen_height()/2.0 + pan2d.y, 3.0, LIME);
                }
            }

            ViewMode::View3D => {
                let delta = mouse_delta_position();
                if is_mouse_button_down(MouseButton::Left) {
                    longitude += delta.x * 3.0; 
                    latitude -= delta.y * 3.0; 
                }

                if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
                    let view_dir = vec3(
                        -longitude.sin() * latitude.cos(),
                        -latitude.sin(),
                        -longitude.cos() * latitude.cos()
                    ).normalize();
                    let up = vec3(0.0, 1.0, 0.0);
                    let right = view_dir.cross(up).normalize();
                    let true_up = right.cross(view_dir).normalize();

                    let pan_speed = zoom * 0.002;
                    target -= right * mouse_delta.x * pan_speed;
                    target += true_up * mouse_delta.y * pan_speed;
                }

                let wheel = mouse_wheel().1;
                if wheel != 0.0 { zoom -= wheel.signum() * (zoom * 0.1); }
                latitude = latitude.clamp(-1.5, 1.5);

                let pos = vec3(
                    target.x + zoom * longitude.sin() * latitude.cos(),
                    target.y + zoom * latitude.sin(), 
                    target.z + zoom * longitude.cos() * latitude.cos()
                );

                set_camera(&Camera3D { position: pos, up: vec3(0.0, 1.0, 0.0), target, ..Default::default() });

                // 3D Wireframe
                for edge in triangulation.undirected_edges() {
                    let [v1, v2] = edge.vertices();
                    let p1 = vec3(v1.data().x as f32, v1.data().z as f32, v1.data().y as f32);
                    let p2 = vec3(v2.data().x as f32, v2.data().z as f32, v2.data().y as f32);
                    let color = if v1.data().z > 0.0 || v2.data().z > 0.0 { GREEN } else { BLUE };
                    draw_line_3d(p1, p2, color);
                }

                // 3D Dots (Spheres)
                for vertex in triangulation.vertices() {
                    let data = vertex.data();
                    let p = vec3(data.x as f32, data.z as f32, data.y as f32);
                    // Drawing a small sphere to represent the point in 3D space
                    draw_sphere(p, 0.05, None, LIME);
                }
            }
        }

        set_default_camera();
        draw_text(&format!("MODE: {:?}", mode), 20.0, 30.0, 20.0, WHITE);

        root_ui().window(hash!("elevation_window"), vec2(20.0, 40.0), vec2(200.0, 50.0), |ui| {
            ui.input_text(hash!("elevation_input"), "Z Elevation", &mut elevation_input);
        });

        if let Ok(val) = elevation_input.parse::<f64>() {
            next_elevation = val;
        }

        next_frame().await
    }
}