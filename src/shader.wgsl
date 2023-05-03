
// Vertex shader
struct VertexIn {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>
}

struct VertexOut {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>
}

@vertex
fn vertex_shader_main(model: VertexIn) -> VertexOut {
	var out: VertexOut;
	out.position = vec4<f32>(model.position, 1.0);
	out.color = model.color;
	return out;
}

// Fragment shader
@fragment
fn fragment_shader_main(in: VertexOut) -> @location(0) vec4<f32> {
	return vec4<f32>(in.color, 1.0);
}
