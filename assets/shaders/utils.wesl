fn saturation_matrix(amount: f32) -> mat4x4<f32> {
    let inv_amount = 1.0 - amount;

    let r = vec4<f32>(0.2126, 0.2126, 0.2126, 0.0) * inv_amount;
    let g = vec4<f32>(0.7152, 0.7152, 0.7152, 0.0) * inv_amount;
    let b = vec4<f32>(0.0722, 0.0722, 0.0722, 0.0) * inv_amount;
    let a = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    return mat4x4<f32>(
        r + vec4<f32>(amount, 0.0, 0.0, 0.0),
        g + vec4<f32>(0.0, amount, 0.0, 0.0),
        b + vec4<f32>(0.0, 0.0, amount, 0.0),
        a
    );
}
