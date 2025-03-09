vec4 blend(vec4 Ca, vec4 Cb) {
    return Cb * Cb.a + Ca * (Ca.a - Cb.a);
}

float avg3(vec3 a) {
    return (a.x + a.y + a.z) / 3.0;
}

float lumi(vec3 a) {
    return sqrt(0.299*(a.r*a.r) + 0.587*(a.g*a.g) + 0.114*(a.b*a.b));
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    vec2 res = iResolution.xy;
    vec2 uv = fragCoord.xy / iResolution.xy;
    vec4 CaBright = vec4(vec3(1.0), 1.0);
    vec4 CaDark = vec4(vec3(0.0), 1.0);
    
    float Cr1P = 0.5;
    // bright
    vec4 Cr1 = texture(iChannel0, uv) * Cr1P + (1.0 - Cr1P);
    // dark
    vec4 Cr2 = texture(iChannel1, uv) * (1.0 - Cr1P);
    
    float af = lumi(Cr2.rgb) + 1.0 - lumi(Cr1.rgb);
    vec3 a = Cr2.rgb + 1.0 - Cr1.rgb;
    vec4 Cb = vec4(Cr2.rgb / af, af);
    
    if (int(iTime) % 2 == 0) {
    Cb = vec4((Cr1.rgb + af - 1.0) / af, af);
    }

    int count = 3;
    int l = int(min(iResolution.x, iResolution.y)) / count * 2;
    int lh = l / 2;
    float off = iTime * 500.0;
    if (int(uv.x * res.x + off) % l < lh != int(uv.y * res.y) % l < lh) {
        fragColor = blend(CaBright, Cb);
    } else {
        fragColor = blend(CaDark, Cb);
    }
}

