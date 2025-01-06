#ifdef GL_ES
precision mediump float;
#endif

#define PI 3.14159265359

#if defined(GL_OES_standard_derivatives)
#extension GL_OES_standard_derivatives : enable
#endif

float aastep(float threshold, float value) {
#if !defined(GL_ES) || __VERSION__ >= 300 || defined(GL_OES_standard_derivatives)
    float afwidth = 0.7 * length(vec2(dFdx(value), dFdy(value)));
    return smoothstep(threshold-afwidth, threshold+afwidth, value);
#elif defined(AA_EDGE)
    float afwidth = AA_EDGE;
    return smoothstep(threshold-afwidth, threshold+afwidth, value);
#else
    return step(threshold, value);
#endif
}

float fill(float x, float size) {
    return 1.0 - step(size, x);
}

#define saturate(V) clamp(V, 0.0, 1.0)

float stroke(float x, float size, float w) {
    float d = step(size, x + w * 0.5) - step(size, x - w * 0.5);
    return saturate(d);
}

vec2 ratio(in vec2 v, in vec2 s) {
    return mix( vec2((v.x*s.x/s.y)-(s.x*.5-s.y*.5)/s.y,v.y),
                vec2(v.x,v.y*(s.y/s.x)-(s.y*.5-s.x*.5)/s.x),
                step(s.x,s.y));
}

vec3 fisheye2xyz(vec2 uv) {
    vec2 ndc = uv * 2.0 - 1.0;
    float R = sqrt(ndc.x * ndc.x + ndc.y * ndc.y);
    vec3 dir = vec3(ndc.x / R, 0.0, ndc.y / R);
    float Phi = (R) * PI * 0.52;
    dir.y   = cos(Phi);//clamp(, MinCos, 1.0);
    dir.xz *= sqrt(1.0 - dir.y * dir.y);
    return dir;
}

uniform float u_time;
uniform vec2 u_resolution;
uniform vec2 u_mouse;

vec3 white = vec3(1.0, 1.0, 1.0);
vec3 black = vec3(0.0, 0.0, 0.0);
vec3 red = vec3(1.0, 0.0, 0.0);
vec3 green = vec3(0.0, 1.0, 0.0);
vec3 blue = vec3(0.0, 0.0, 1.0);
vec3 yellow = mix(red, green, vec3(0.0, 1.0, 0.0));
vec3 light_blue = mix(green, blue, vec3(0.0, 0.0, 1.0));
vec3 purple = mix(blue, red, vec3(1.0, 0.0, 0.0));

vec3 redA = vec3(213.0, 43.0, 30.0) / 256.0;
vec3 blueA = vec3(0.0, 57.0, 166.0) / 256.0;
vec3 yellowA = vec3(255.0, 205.0, 1.0) / 256.0;

float plot(vec2 st, float pct) {
  return smoothstep(pct - 0.02, pct, st.y) -
  smoothstep(pct, pct + 0.02, st.y);
}

float doubleCubicSeat (float x, float a, float b){

  float epsilon = 0.00001;
  float min_param_a = 0.0 + epsilon;
  float max_param_a = 1.0 - epsilon;
  float min_param_b = 0.0;
  float max_param_b = 1.0;
  a = min(max_param_a, max(min_param_a, a));
  b = min(max_param_b, max(min_param_b, b));

  float y = 0.0;
  if (x <= a){
    y = b - b*pow(1.0-x/a, 3.0);
  } else {
    y = b + (1.0-b)*pow((x-a)/(1.0-a), 3.0);
  }
  return y;
}

float easeInOutCubic(float x) {
  if (x < 0.5) {
    return 4.0 * pow(x, 3.0);
  } else {
    return 1.0 - pow(-2.0 * x + 2.0, 3.0) / 2.0;
  }
}

float easeInOutSine(float x) {
  return -(cos(PI * x) - 1.0) / 2.0;
}

vec3 rf_flag_amim(float t, vec2 st) {
  vec3 rf_flag = vec3(0.0);
  vec3 empire_flag = vec3(0.0);
  vec3 color = vec3(0.0);
 
  float pct1 = step(0.33, st.y);
  float pct2 = step(0.66, st.y);

  rf_flag = mix(redA, blueA, pct1);
  rf_flag = mix(rf_flag, white, pct2);

  empire_flag = mix(white, yellowA, pct1);
  empire_flag = mix(empire_flag, black, pct2);

  return mix(rf_flag, empire_flag, easeInOutSine(t));
}

vec3 rainbow(vec2 st) {
  vec3 color = vec3(0.0);

  vec3 orange = mix(red, yellow, vec3(0.0, 0.5, 0.0));

  float pct = smoothstep(0.0, 1.0 / 6.0, st.x);
  color = mix(red, orange, pct);

  pct = smoothstep(1.0 / 6.0, 2.0 / 6.0, st.x);
  color = mix(color, yellow, pct);

  pct = smoothstep(2.0 / 6.0, 3.0 / 6.0, st.x);
  color = mix(color, green, pct);

  pct = smoothstep(3.0 / 6.0, 4.0 / 6.0, st.x);
  color = mix(color, light_blue, pct);

  pct = smoothstep(4.0 / 6.0, 5.0 / 6.0, st.x);
  color = mix(color, blue, pct);

  pct = smoothstep(5.0 / 6.0, 1.0, st.x);
  color = mix(color, purple, pct);

  return color;
}

vec3 rgb2hsb(in vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz),
               vec4(c.gb, K.xy),
               step(c.b, c.g));

  vec4 q = mix(vec4(p.xyw, c.r),
               vec4(c.r, p.yzx),
               step(p.x, c.r));

  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;

  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)),
              d / (q.x + e),
              q.x);
}

vec3 hsb2rgb_smooth(in vec3 c) {
  vec3 rgb = clamp(abs(mod(c.x * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0,
                   0.0,
                   1.0);
  rgb = rgb * rgb * (3.0 - 2.0 * rgb);
  return c.z * mix(vec3(1.0), rgb, c.y);
}

vec3 hsb2rgb(in vec3 c) {
  vec3 rgb = clamp(abs(mod(c.x * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0,
                   0.0,
                   1.0);
  return c.z * mix(vec3(1.0), rgb, c.y);
}

float hueShift(float hue, float a) {
  hue = hue * PI * 2.0 + a;
  return fract(hue / (PI * 2.0));
}

float tone(float x, float k) {
  return (k + 1.0) / (1.0 + k * x);
}

float quadraticBezier (float x, float a, float b){
  // adapted from BEZMATH.PS (1993)
  // by Don Lancaster, SYNERGETICS Inc. 
  // http://www.tinaja.com/text/bezmath.html

  float epsilon = 0.00001;
  a = max(0.0, min(1.0, a));
  b = max(0.0, min(1.0, b));
  if (a == 0.5){
    a += epsilon;
  }
  
  // solve t from x (an inverse operation)
  float om2a = 1.0 - 2.0*a;
  float t = (sqrt(a*a + om2a*x) - a)/om2a;
  float y = (1.0-2.0*b)*(t*t) + (2.0*b)*t;
  return y;
}

vec3 polar_hsb(vec2 st, float t) {
  vec2 toCenter = vec2(0.5) - st;
  toCenter *= 1.1;

  float x = toCenter.x;
  float y = toCenter.y;
  x = sqrt(1.0 - pow(x - 1.0, 2.0));
  y = sqrt(1.0 - pow(y - 1.0, 2.0));
  float rot_x = x * cos(t) - y * sin(t);
  float rot_y = y * cos(t) + x * sin(t);
  float angle = atan(y, x);
  float radius = length(toCenter) * 4.0;
  float two_pi = PI * 2.0;
  float hue = (angle / two_pi) + 0.5;
  return hsb2rgb_smooth(vec3(hue, radius, 1.0));
}

vec3 smooth_rect(vec2 st, float border, float blur_len) {
  float blur_end = border + blur_len;
    float left = smoothstep(border, blur_end, st.x);
    float right = smoothstep(border, blur_end, 1.0 - st.x);
    float top = smoothstep(border, blur_end, st.y);
    float bottom = smoothstep(border, blur_end, 1.0 - st.y);

    return vec3(left * bottom * right * top);
}

vec3 rectangle(vec2 st) {
    float left = step(0.1, st.x);
    float right = step(0.1, 1.0 - st.x);
    float top = step(0.1, st.y);
    float bottom = step(0.1, 1.0 - st.y);

    return vec3(left * bottom * right * top);
}

vec3 rectangle_border(vec2 st, float border_start, float width) {
  vec3 color = vec3(0.0);
  float border_end = border_start + width;
  float left = step(border_start, st.x);
  float left_end = step(border_end, st.x);
  left += left_end;

  float right = step(border_start, 1.0 - st.x);
  float right_end = step(border_end, 1.0 - st.x);
  right += right_end;

  float bottom = step(border_start, st.y);
  float bottom_end = step(border_end, st.y);
  bottom += bottom_end;

  float top = step(border_start, 1.0 - st.y);
  float top_end = step(border_end, 1.0 - st.y);
  top += top_end;

  color = 1.0 - vec3(left * right * bottom * top);
  color = color + vec3(left_end * right_end * bottom_end * top_end);

  return color;
}

float rectSDF(vec2 p, vec2 b, float r) {
    vec2 d = abs(p - 0.5) * 4.2 - b + vec2(r);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - r;
}

float rectSDF(vec2 st, vec2 s) {
  st = st * 2.0 - 1.0;
  float x_d = abs(st.x / s.x);
  float y_d = abs(st.y / s.y);
  float sdf = max(x_d, y_d);

  return sdf;
}

void main() {
  vec2 st = gl_FragCoord.xy / u_resolution;
  vec4 color = vec4(vec3(0.0), 1.0);
  st = ratio(st, u_resolution);

  vec2 p = st + vec2(0.1, 0.1);

  float sdf = 0.0;
  sdf = rectSDF(st, vec2(0.1));
  color.rgb = vec3(fill(sdf, 1.0));

  gl_FragColor = color;
}
