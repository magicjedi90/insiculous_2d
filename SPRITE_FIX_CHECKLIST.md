# ✅ Sprite Fix Checklist

Based on debug findings, here are exact steps to fix:

## Step 1: Verify Vertex Format (DONE - needs verification)
- [ ] Add to `SpriteVertex::desc()`: 
  ```rust
  log::info!("SpriteVertex size: {}", std::mem::size_of::<SpriteVertex>());
  log::info!("  position offset: 0");
  log::info!("  tex_coords offset: {}", std::mem::size_of::<[f32; 3]>());
  log::info!("  color offset: {}", std::mem::size_of::<[f32; 5]>());
  ```
- [ ] Add to `SpriteInstance::desc()`:
  ```rust
  log::info!("SpriteInstance size: {}", std::mem::size_of::<SpriteInstance>());
  ```
- [ ] Compare logged sizes with shader expectations

## Step 2: Add Instance Data Debug Shader
- [ ] Create `debug_instance.wgsl` that uses `instance.world_position` directly:
  ```wgsl
  out.clip_position = vec4<f32>(instance.world_position, 0.0, 1.0);
  ```
- [ ] If sprites appear at center, instance data is being consumed
- [ ] If sprites don't appear, instance buffer binding is wrong

## Step 3: Add Camera Debug Shader
- [ ] Create `debug_camera.wgsl` that applies camera transform to fixed position:
  ```wgsl
  let pos = camera.view_projection * vec4<f32>(0.0, 0.0, 0.0, 1.0);
  out.clip_position = pos;
  ```
- [ ] If sprite appears, camera matrix is correct
- [ ] If not, camera math or uniforms are wrong

## Step 4: Validate Index Buffer
- [ ] Log index buffer data: [0, 1, 2, 0, 2, 3]
- [ ] Try `draw()` instead of `draw_indexed()` with 6 vertices
- [ ] If works, index buffer is the issue

## Step 5: Validate Instance Count
- [ ] Log `instance_count` and `instance_offset`:
  ```rust
  log::info!("Drawing batch: instance_offset={}, instance_count={}", instance_offset, instance_count);
  ```
- [ ] Verify not drawing zero instances

## Step 6: Check Vertex Buffer Content
- [ ] Insert debugger in render loop to dump vertex data:
  ```rust
  let vertices = [
      SpriteVertex::new(Vec3::new(-0.5, 0.5, 0.0), Vec2::new(0.0, 0.0), Vec4::ONE),
      // ...
  ];
  log::info!("Vertices: {:?}", vertices);
  ```

## Expected Fix
One of these will work:
1. Change `array_stride` in `SpriteInstance::desc()`
2. Adjust attribute offsets in vertex layout
3. Fix camera matrix calculation
4. Change `draw_indexed()` to not use indexing
5. Fix shader location indices
