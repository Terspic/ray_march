#version 450

layout(location = 0) in vec2 o_uv;
layout(location = 0) out vec4 frag_color;

layout(set=0, binding=0) uniform texture2D u_texture;
layout(set=0, binding=1) uniform sampler u_sampler;

void main() {
    frag_color = texture(sampler2D(u_texture, u_sampler), o_uv);
}