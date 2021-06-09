# Bevy shadow
Simply adds shadows to directional lights in the bevy engine.

## Usage
To use simply add `ShadowPlugin` to your app and `Shadowless` to anything that shouldn't cast a shadow.

## Configuration
For configuration there are several options in the plugin.
```rust
pub struct ShadowPlugin {
    /// Resolution of directional light shadow maps.
    pub directional_light_resolution: u32,
    /// If true, replaces the default pbr pipeline.
    /// If false use [`prelude::SHADOW_PBR_PIPELINE`].
    pub replace_pbr_pipeline: bool,
    /// If false, the shadow pbr pipeline won't be created.
    /// Disable if you want to implement your own.
    pub create_pbr_pipeline: bool,
    /// If false then the shadow pass won't be connected to main pass.
    pub connect_to_main_pass: bool,
}
```
There is also a configuration component for every light, that can optionally be inserted.
```rust
pub struct ShadowDirectionalLight {
    /// Size of the area covered by the light. 
    /// Everything outside will be lit by default.
    pub size: f32,
    /// Near plane of projection.
    pub near: f32,
    /// Far plane of projection.
    pub far: f32,
}
```

## Compatibility
Currently only targets main.