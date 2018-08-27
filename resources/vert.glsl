#version 330

layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 color_in;
layout(location = 2) in vec2 tex_coord_in;

out vec3 color_frag;
out vec2 tex_coord;

void main()
{
	color_frag = color_in;
	tex_coord = tex_coord_in;
	gl_Position = vec4(pos.x,pos.y,0.0,1.0);
}