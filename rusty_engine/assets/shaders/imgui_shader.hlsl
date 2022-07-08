struct VSInput
{
    float2 Position : POSITION;
    float2 UV       : TEX_COORD;
    float4 Color    : COLOR;
};

struct PSInput
{
    float4 Position : SV_POSITION;
    float2 UV       : TEX_COORD;
    float4 Color    : COLOR;
};

cbuffer VSSystemUniforms : register(b0)
{
    float4x4 sys_MVPMatrix;
};

PSInput VSMain(VSInput input)
{
    PSInput output;
    output.Position = mul(sys_MVPMatrix, float4(input.Position.xy, 0.f, 1.f));
    output.Color = input.Color;
    output.UV  = input.UV;
    return output;
}

SamplerState u_Sampler : register(s0);
Texture2D u_Texture : register(t0);

float4 PSMain(PSInput input) : SV_Target
{
    return input.Color * u_Texture.Sample(u_Sampler, input.UV); 
}