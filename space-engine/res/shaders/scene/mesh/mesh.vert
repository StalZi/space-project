#version 460
#extension GL_EXT_buffer_reference : require

layout (location = 0) out vec3 outColor;
layout (location = 1) out vec2 outUV;

struct Vertex {
    vec3 position;
    vec3 normal;
    vec4 color;
    vec2 uv;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer{ 
	Vertex vertices[];
};

//push constants block
layout( push_constant ) uniform constants {	
	mat4 mvp;
	VertexBuffer vertexBuffer;
} PushConstants;

void main() 
{	
	//load vertex data from device address
	Vertex v = PushConstants.vertexBuffer.vertices[gl_VertexIndex];

	//output data
	gl_Position = PushConstants.mvp * vec4(v.position, 1.0f);
	outColor = (v.normal * 0.5 + 0.5);
	outUV = v.uv;
}