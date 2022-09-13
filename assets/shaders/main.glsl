#version 450

#include "utils.glsl"
#include "sdf.glsl"

// output image
layout(set=0, binding=0, rgba8)
writeonly uniform image2D u_output;

// uniforms
layout(set=0, binding=1)
uniform Uniforms {
	vec3 u_eye;
	float u_fov;
	vec3 u_target;
	float u_time;
};

// constants
const float MAX_STEPS = 256; 
const float MIN_HIT_DIST = 0.001;
const float MAX_DIST = 100.0;

#define AA 4
#define BACKGROUND_ENABLE 1
#define SHADOW_ENABLED 1

const Material[] materials = {
	// background materials
	Material(
		vec3(1.0, 0.0, 0.0),
		vec3(1.0, 0.0, 0.0),
		vec3(0.0),
		1.0
	),	
	Material(
		vec3(0.0, 1.0, 0.0),
		vec3(0.0, 1.0, 0.0),
		vec3(0.0),
		1.0
	),
	Material(
		vec3(0.0, 0.0, 1.0),
		vec3(0.0, 0.0, 1.0),
		vec3(0.0),
		1.0
	),

	// main scene materials
	Material(
		vec3(0.8, 0.1, 0.08),
		vec3(0.45, 0.02, 0.05),
		vec3(0.05, 0.05, 0.05),
		12.0
	),
	Material(
		vec3(0.8, 0.7, 0.5),
		vec3(0.2, 0.3, 0.4),
		vec3(0.0),
		1.0
	)
};

Hit background_map(vec3 p) {
	float bar_length = MAX_DIST;
	Hit x_axis = Hit(sdInfiniteCylinder(p, vec3(0.0), vec3(1.0, 0.0, 0.0), 0.03), 0);
	Hit y_axis = Hit(sdInfiniteCylinder(p, vec3(0.0), vec3(0.0, 1.0, 0.0), 0.03), 1);
	Hit z_axis = Hit(sdInfiniteCylinder(p, vec3(0.0), vec3(0.0, 0.0, 1.0), 0.03), 2);

	return opUnion(opUnion(x_axis, y_axis), z_axis);
}

Hit scene(vec3 p) {
	Hit sphere1 = Hit(sdSphere(p, vec3(0.0, 1.1, 0.0), 1.0), 3);
	Hit box1 = Hit(sdBox(p, vec3(10.0, 0.1, 10.0)), 4);
	return opUnion(sphere1, box1);
}

Hit map(vec3 p) {
#if BACKGROUND_ENABLE
	return opUnion(scene(p), background_map(p));
#else
	return scene(p);
#endif
}

vec3 get_normal(vec3 p) {
	// small step
	const vec3 h = vec3(MIN_HIT_DIST, 0.0, 0.0);

	// compute gradient coordinates
	float fp = map(p).dist;
	float gx = map(p + h.xyy).dist - fp;
	float gy = map(p + h.yxy).dist - fp;
	float gz = map(p + h.yyx).dist - fp;

	// normalize gradient to get the normal
	return normalize(vec3(gx, gy, gz));
}

Hit ray_cast(vec3 ro, vec3 rd) {
	Hit t = Hit(0.001, 0);
	for (int i = 0; i < MAX_STEPS; i++) {
		Hit d = map(ro + rd * t.dist);
		if (d.dist <= MIN_HIT_DIST || t.dist >= MAX_DIST) {
			break;
		}
		t.dist += d.dist;
		t.id = d.id;
	}

	return t;
}

// adapt from soft shadows (https://iquilezles.org/www/articles/rmshadows/rmshadows.htm)
float shadow(vec3 ro, vec3 rd, float tmax, float k) {
	float res = 1.0;
	for (float t = MIN_HIT_DIST; t < tmax;) {
		float h = scene(ro + rd*t).dist;
		if (h<MIN_HIT_DIST) {
			return 0.0;
		}
		res = min(res, k*h/t);
		t += h;
	}

	return res;
}

vec3 compute_lighting(vec3 ro, vec3 rd, vec3 pos, vec3 normal, vec3 light_pos, int mat_id) {
	Material mat = materials[mat_id];

	vec3 light_dir = normalize(light_pos - pos);
	vec3 view_dir = normalize(ro - pos);
	vec3 reflect_dir = reflect(-light_dir, normal);

	float dif = max(dot(normal, light_dir), 0.0);
	float amb = 0.5 + 0.4*dot(normal, vec3(0.0, 1.0, 0.0));
	float spec = pow(max(dot(view_dir, reflect_dir), 0.0), mat.specular_exponent);

	// shadows
#if SHADOW_ENABLED
	if (dif > 0.001) {
		dif *= shadow(pos + normal * 0.001, light_dir, length(light_pos - pos), 32.0);
	}
#endif	

	return mat.diffuse * dif + mat.ambient * amb + mat.specular * spec;
}

vec3 background_color(vec3 ro, vec3 rd) {
	return vec3(0.0);
}

vec3 compute_color(vec3 ro, vec3 rd) {
	Hit t = ray_cast(ro, rd);

	if (t.dist >= MAX_DIST) {
		return background_color(ro, rd);
	}

	vec3 pos = ro + t.dist * rd;
	vec3 normal = get_normal(pos);
	vec3 light_pos = vec3(5.0 * sin(u_time * 0.5), 5.0, 5.0 * cos(u_time * 0.5));

	return compute_lighting(ro, rd, pos, normal, light_pos, t.id);
}

layout(local_size_x = 16, local_size_y = 16) in;
void main() {
	// translate coordinates from pixel
	ivec2 coords = ivec2(gl_GlobalInvocationID.xy);
	ivec2 resolution = imageSize(u_output);
	vec2 uv = map_pixel_to_screen(coords, resolution);
	mat3 camera = build_camera(u_eye, u_target);

// enable antialiasing
# if AA>1
	vec3 final_color = vec3(0.0);
	for(int j = 0; j < AA; j++){
		for (int i = 0; i < AA; i++) {
			// offset for antialiasing
			vec2 o = (vec2(i, j) / AA - 0.5) / resolution;
			vec3 rd = get_ray_dir(camera, u_fov, uv.x + o.x, uv.y + o.y);
			final_color += compute_color(u_eye, rd);
		}
	}
	final_color /= float(AA * AA); // average color
// no antialiasing
#else
	Ray r = Ray(u_eye, get_ray_dir(camera, u_fov, uv.x, uv.y));
	vec3 final_color = compute_color(r.orig, r.dir);
#endif

	// record pixel
	imageStore(u_output, coords, vec4(final_color, 1.0));
}
