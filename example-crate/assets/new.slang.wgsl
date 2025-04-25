struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) dispatchThreadID_0 : vec2<u32>,
    @location(1) screenSize_0 : vec2<i32>,
};

@fragment
fn imageMain( _S1 : pixelInput_0) -> pixelOutput_0
{
    var _S2 : pixelOutput_0 = pixelOutput_0( vec4<f32>(0.30000001192092896f, 0.69999998807907104f, 0.55000001192092896f, 1.0f) );
    return _S2;
}

