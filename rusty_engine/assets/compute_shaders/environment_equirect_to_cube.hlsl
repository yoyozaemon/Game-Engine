static const float PI = 3.141592;
static const float TwoPI = 2 * PI;

Texture2D inputTexture : register(t0);
RWTexture2DArray<float4> outputTexture : register(u0);

SamplerState defaultSampler : register(s0);

float3 getSamplingVector(uint3 ThreadID)
{
	float outputWidth, outputHeight, outputDepth;
	outputTexture.GetDimensions(outputWidth, outputHeight, outputDepth);

    float2 st = ThreadID.xy/float2(outputWidth, outputHeight);
    float2 uv = 2.0 * float2(st.x, st.y) - 1.0;

	float3 ret = float3(0.0, 0.0, 0.0);
	switch(ThreadID.z)
	{
	case 0: ret = float3(1.0,  uv.y, -uv.x); break;
	case 1: ret = float3(-1.0, uv.y,  uv.x); break;
	case 2: ret = float3(uv.x, -1.0, uv.y); break;
	case 3: ret = float3(uv.x, 1.0, -uv.y); break;
	case 4: ret = float3(uv.x, uv.y, 1.0); break;
	case 5: ret = float3(-uv.x, uv.y, -1.0); break;
	}
    return normalize(ret);
}

[numthreads(32, 32, 1)]
void CSMain(uint3 ThreadID : SV_DispatchThreadID)
{
	float3 v = getSamplingVector(ThreadID);
	
	float phi   = atan2(v.z, v.x);
	float theta = acos(v.y);

	float4 color = inputTexture.SampleLevel(defaultSampler, float2(phi/TwoPI, theta/PI), 0);

	outputTexture[ThreadID] = color;
}