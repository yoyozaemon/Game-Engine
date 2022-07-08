static const float gamma     = 2.2;
static const float pureWhite = 1.0;

struct VSInput
{
    float3 Position : POSITION;
    float2 UV       : TEX_COORD;
};

struct PSInput
{
    float4 PositionSV : SV_POSITION;
    float2 UV         : TEX_COORD;
};

PSInput VSMain(in VSInput input)
{
    PSInput output;
    output.UV         = input.UV;
    output.PositionSV = float4(input.Position, 1.0);
    return output;
}

cbuffer PSMaterialUniforms : register(b0)
{
    float u_Exposure;
};

Texture2D u_SceneTexture : register(t0);
SamplerState u_Sampler: register(s0);

float4 PSMain(in PSInput input) : SV_Target
{
    float3 color = u_SceneTexture.Sample(u_Sampler, input.UV).rgb * u_Exposure;
	
	float luminance = dot(color, float3(0.2126, 0.7152, 0.0722));
	float mappedLuminance = (luminance * (1.0 + luminance / (pureWhite * pureWhite))) / (1.0 + luminance);

	float3 mappedColor = (mappedLuminance / luminance) * color;

	return float4(pow(abs(mappedColor), 1.0 / gamma), 1.0);
}