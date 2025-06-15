## Vulkan SDK 설정(맥)
```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```

## GLSL 컴파일
```
glslc src/shaders/test.vert -o src/shaders/test.vert.spv
glslc src/shaders/test.frag -o src/shaders/test.frag.spv
```

```
glslc src/shaders/final.vert -o src/shaders/final.vert.spv
glslc src/shaders/final.frag -o src/shaders/final.frag.spv
```

```
glslc src/shaders/shadow.vert -o src/shaders/shadow.vert.spv
glslc src/shaders/geometry.vert -o src/shaders/geometry.vert.spv
glslc src/shaders/geometry.frag -o src/shaders/geometry.frag.spv
```
