#version 330

out vec4 color;

in vec3 color_frag;
in vec2 tex_coord;
in float enable_texture;

uniform sampler2D art;

void main()
{
	if(enable_texture > 0.5) {
		color = vec4(color_frag.r,color_frag.g,color_frag.b,1.0)*texture(art,tex_coord);
	} else {
		color = vec4(color_frag.r,color_frag.g,color_frag.b,1.0);
	}
}