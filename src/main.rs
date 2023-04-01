mod marlin;

use winit::dpi::LogicalSize;
use marlin::{Marlin, Entity, Point};

#[tokio::main]
async fn main() {

    let mut window = Marlin::new(400.0, 400.0).await;
    window.state().window().set_title("Marlin");
    
    let tri = Entity::from_points(&window, vec![
        Point::new(200.0, 100.0, [1.0, 0.0, 0.0]),
        Point::new(100.0, 200.0, [1.0, 0.0, 0.0]),
        Point::new(0.0, 200.0, [1.0, 0.0, 0.0]),

    ]);

    let weird = Entity::from_points(&window, vec![
        Point::new(1.0, 0.5, [1.0, 0.0, 0.0]),
        Point::new(-0.5, -0.5, [0.0, 1.0, 0.0]),
        Point::new(0.5, -0.5, [0.0, 0.0, 1.0]),
    ]);

    window.add_entity(tri);
    // window.add_entity(weird);
    window.draw_point(0.5, 0.5, [1.0, 0.5, 0.2]);

    window.run().await;

}
