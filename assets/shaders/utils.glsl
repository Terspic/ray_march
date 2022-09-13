struct Ray {
	vec3 orig;
	vec3 dir;
};

struct Material {
	vec3 diffuse;
	vec3 ambient;
	vec3 specular;
	float specular_exponent;
};

struct Hit {
	float dist;
	int id;
};

mat3 build_camera(vec3 eye, vec3 target) {
	vec3 w = normalize(target - eye);
	vec3 u = normalize(cross(w, vec3(0.0, 1.0, 0.0)));
	vec3 v = cross(u, w);

	return mat3(u, v, w);
}

// shoot ray form pixel coordinates
vec3 get_ray_dir(mat3 camera, float fov, float u, float v) {
	return camera * normalize(vec3(u, v, fov));
}

vec2 map_pixel_to_screen(ivec2 coords, vec2 dim) {
	vec2 uv =  (2.0 * coords - dim.xy) / dim.y;
	return vec2(uv.x, -uv.y);
}

float checker(vec2 p) {
	vec2 q = floor(p);
	return mod(q.x+q.y, 2);
}
