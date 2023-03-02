mod marlin;

use marlin::{Marlin, Entity, Vertex};

#[tokio::main]
async fn main() {

    let mut window = Marlin::new().await;

    let tri = Entity::from_points(vec![
        Vertex::new(0.0, 0.5, 0.0, [1.0, 0.0, 0.0]),
        Vertex::new(-0.5, -0.5, 0.0, [0.0, 1.0, 0.0]),
        Vertex::new(0.5, -0.5, 0.0, [0.0, 0.0, 1.0]),
    ]);

    let weird = Entity::from_points(vec![
        Vertex::new(1.0, 0.5, 0.0, [1.0, 0.0, 0.0]),
        Vertex::new(-0.5, -0.5, 0.0, [0.0, 1.0, 0.0]),
        Vertex::new(0.5, -0.5, 0.0, [0.0, 0.0, 1.0]),
    ]);

    window.add_entity(tri);
    // window.add_entity(weird);

    window.run().await;

}
