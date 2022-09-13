float sdSphere(vec3 p, vec3 c, float r) {
	return length(p - c) - r;
}

float sdPlane(vec3 p, vec3 normal, float o) {
	return dot(p, normal) - o;
}

float sdXZPlane(vec3 p, float yoffset) { 
	return p.y - yoffset;
}

float sdXYPlane(vec3 p, float zoffset) {
	return p.z - zoffset;
}

float sdYZPlane(vec3 p, float xoffset) {
	return p.x - xoffset;
}

float sdBox(vec3 p, vec3 b) {
	vec3 q = abs(p) - b;
	return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdCapsule(vec3 p, vec3 a, vec3 b, float r) {
	vec3 ap = p - a; vec3 ab = b - a;
	float h = clamp(dot(ap, ab) / dot(ab, ab), 0.0, 1.0);
	return length(ap - h * ab) - r;
}

float sdCapsule(vec3 p, float d, float r) {
	p.x -= clamp(p.x, 0.0, d);
	return length(p) - r;
}

float sdInfiniteCylinder(vec3 p, vec3 pos, vec3 dir, float r) {
	vec3 q = p - pos;
	float d = length(cross(q, dir)) / length(dir);

	return d - r;
}

float sdCylinder(vec3 p, float h, float r) {
	return 0.0;
}

Hit opUnion(Hit d1, Hit d2) {
	return (d1.dist < d2.dist) ? d1 : d2;
}

Hit opIntersect(Hit d1, Hit d2) {
	return (d1.dist > d2.dist) ? d1 : d2;
}

Hit opSubstract(Hit d1, Hit d2) {
	return (d1.dist > -d2.dist) ? d1 : Hit(-d2.dist, d2.id);
}

vec3 opTx(vec3 p, vec3 translation) {
	return p - translation;
}

vec3 opRotateX(vec3 p, float angle) {
	float c = cos(-angle);
	float s = sin(-angle);
	mat3 rot = 
		mat3(1, 0, 0, 
			 0, c, -s,
			 0, s, c
		);

	return rot * p;
}

vec3 opRotateY(vec3 p, float angle) {
	float c = cos(-angle);
	float s = sin(-angle);
	mat3 rot = 
		mat3(c, 0, -s, 
			 0, 1, 0,
			 s, 0, c
		);

	return rot * p;
}

vec3 opRotateZ(vec3 p, float angle) {
	float c = cos(-angle);
	float s = sin(-angle);
	mat3 rot = 
		mat3(c, -s, 0, 
			 s,  c, 0,
			 0,  0, 1
		);

	return rot * p;
}
