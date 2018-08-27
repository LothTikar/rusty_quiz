#version 330

out vec4 color;

in vec3 color_frag;

void main()
{
	color = vec4(color_frag.r,color_frag.g,color_frag.b,1.0);
}