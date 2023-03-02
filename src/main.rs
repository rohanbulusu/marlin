mod marlin;

use marlin::{Vertex, Entity};


#[tokio::main]
async fn main() {

    let tri = Entity::from_points(vec![
        Vertex::new(0.0, 0.5, 0.0, [1.0, 0.0, 0.0]),
        Vertex::new(-0.5, -0.5, 0.0, [0.0, 1.0, 0.0]),
        Vertex::new(0.5, -0.5, 0.0, [0.0, 0.0, 1.0])
    ]);

    marlin::run(vec![tri]).await;
}
