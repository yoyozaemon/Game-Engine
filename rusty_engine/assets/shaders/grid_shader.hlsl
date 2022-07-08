struct VSInput
{
    float3 Position : POSITION;
    float2 UV       : TEX_COORD;
    float3 Normal   : NORMAL;
    float3 Tangent  : TANGENT;
    float3 Bitangent: BITANGENT;
};

struct PSInput
{
    float4 PositionSV : SV_POSITION;
    float3 Position   : POSITION;
    float2 UV         : TEX_COORD;
};

cbuffer VSSystemUniforms : register(b0)
{
    matrix sys_ProjectionMatrix;
    matrix sys_ViewMatrix;
    matrix sys_Transform;
    float3 sys_CameraPosition;
};

PSInput VSMain(in VSInput input)
{
    PSInput output;
    output.Position = mul(float4(input.Position, 1.0), sys_Transform).xyz;
    output.UV = input.UV;
    output.PositionSV = mul(float4(output.Position, 1.0), mul(sys_ViewMatrix, sys_ProjectionMatrix));
    return output;
}

Texture2D u_GridTexture : register(t0);
SamplerState u_Sampler: register(s0);

float4 PSMain(in PSInput input) : SV_Target
{
    return u_GridTexture.Sample(u_Sampler, input.UV);
}