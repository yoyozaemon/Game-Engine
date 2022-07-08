struct VSInput
{
	float3 Position : POSITION;
    float2 UV 		: TEX_COORD;
};

struct VSOutput
{
	float4 PositionCS : SV_POSITION;
	float3 Position	  : POSITION;
	float2 UV 		  : TEX_COORD;
};

cbuffer VSSystemUniforms : register(b0)
{
    matrix sys_ProjectionMatrix;
    matrix sys_ViewMatrix;
    matrix sys_Transform;
    float3 sys_CameraPosition;
};

cbuffer VSMaterialUniforms : register(b1)
{
	matrix u_InvViewProjMatrix;
}

VSOutput VSMain(in VSInput input)
{
	VSOutput output;
	float3 pos = input.Position;
	pos.z = 1.0f;
	output.PositionCS = float4(pos, 1.0);
	output.Position = mul(float4(pos, 1.0), (float4x3)u_InvViewProjMatrix).xyz;
	output.UV = input.UV;
	return output;
}

TextureCube u_EnvironmentMap : register(t0);
SamplerState u_EnvironmentMapSampler : register(s0);

float4 PSMain(in VSOutput input) : SV_TARGET
{
	return u_EnvironmentMap.SampleLevel(u_EnvironmentMapSampler, input.Position, 0);
}