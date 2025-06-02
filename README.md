# eren_next
다음 버전의 에렌엔진을 작업하는 소스코드 저장소입니다.

* TypeScript 버전의 경우, Rust 버전을 백엔드로 사용하고 TypeScript로는 로직을 구현합니다.
* Rust Wasm 타겟은 wgpu를 사용하며 WGSL로 쉐이더 코드를 작성합니다.
* 기타 타겟은 ash를 사용하며 GLSL로 쉐이더 코드를 작성합니다.

```
export VULKAN_SDK=$HOME/VulkanSDK/1.4.313.1/macOS
export DYLD_FALLBACK_LIBRARY_PATH=$VULKAN_SDK/lib
export VK_ICD_FILENAMES=$VULKAN_SDK/share/vulkan/icd.d/MoltenVK_icd.json
export VK_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
```

## TODO
- WGPU 환경에서도 단일 렌더 패스로 동작하게 변경

## 주의사항
- macOS나 iOS에서는 MoltenVK 레이어 때문에 Vulkan(Ash) 방식이 네이티브 Metal(WGPU)보다 느립니다.
