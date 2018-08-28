#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 color_in;
layout(location = 2) in vec2 tex_coord_in;
layout(location = 3) in float tex_enable_in;

out vec3 color_frag;
out vec2 tex_coord;
out float enable_texture;

void main()
{
	enable_texture = tex_enable_in;
	color_frag = color_in;
	tex_coord = tex_coord_in;
	gl_Position = vec4(pos.x,pos.y,pos.z,1.0);
}