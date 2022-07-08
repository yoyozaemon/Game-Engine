static const float PI = 3.141592;
static const float TwoPI = 2 * PI;
static const float Epsilon = 0.001;

static const uint NumSamples = 512;
static const float InvNumSamples = 1.0 / float(NumSamples);

RWTexture2D<float2> LUT : register(u0);

float radicalInverse_VdC(uint bits)
{
	bits = (bits << 16u) | (bits >> 16u);
	bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
	bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
	bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
	bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
	return float(bits) * 2.3283064365386963e-10;
}

float2 sampleHammersley(uint i)
{
	return float2(i * InvNumSamples, radicalInverse_VdC(i));
}

float3 sampleGGX(float u1, float u2, float roughness)
{
	float alpha = roughness * roughness;

	float cosTheta = sqrt((1.0 - u2) / (1.0 + (alpha*alpha - 1.0) * u2));
	float sinTheta = sqrt(1.0 - cosTheta*cosTheta);
	float phi = TwoPI * u1;

	return float3(sinTheta * cos(phi), sinTheta * sin(phi), cosTheta);
}

float gaSchlickG1(float cosTheta, float k)
{
	return cosTheta / (cosTheta * (1.0 - k) + k);
}

float gaSchlickGGX_IBL(float cosLi, float cosLo, float roughness)
{
	float r = roughness;
	float k = (r * r) / 2.0;
	return gaSchlickG1(cosLi, k) * gaSchlickG1(cosLo, k);
}

[numthreads(32, 32, 1)]
void CSMain(uint2 ThreadID : SV_DispatchThreadID)
{
	float outputWidth, outputHeight;
	LUT.GetDimensions(outputWidth, outputHeight);

	float cosLo = ThreadID.x / outputWidth;
	float roughness = ThreadID.y / outputHeight;

	cosLo = max(cosLo, Epsilon);

	float3 Lo = float3(sqrt(1.0 - cosLo*cosLo), 0.0, cosLo);

	float DFG1 = 0;
	float DFG2 = 0;

	for(uint i=0; i<NumSamples; ++i) {
		float2 u = sampleHammersley(i);

		float3 Lh = sampleGGX(u.x, u.y, roughness);

		float3 Li = 2.0 * dot(Lo, Lh) * Lh - Lo;

		float cosLi   = Li.z;
		float cosLh   = Lh.z;
		float cosLoLh = max(dot(Lo, Lh), 0.0);

		if(cosLi > 0.0) {
			float G  = gaSchlickGGX_IBL(cosLi, cosLo, roughness);
			float Gv = G * cosLoLh / (cosLh * cosLo);
			float Fc = pow(1.0 - cosLoLh, 5);

			DFG1 += (1 - Fc) * Gv;
			DFG2 += Fc * Gv;
		}
	}

	LUT[ThreadID] = float2(DFG1, DFG2) * InvNumSamples;
}